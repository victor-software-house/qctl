mod corpus;
pub(crate) mod order;

use crate::cli::LedgerArgs;
use crate::report::{Report, Task};
use anyhow::{Context, Result, bail, ensure};
use garde::Validate;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::schema::ArchiveOrder;
pub use crate::schema::{Ledger, VERSION};

/// The schema qctl was built with, generated from [`crate::schema`].
pub const CANONICAL_SCHEMA: &str = include_str!("../schema/tasks.schema.json");

#[must_use]
pub fn resolve_path(args: &LedgerArgs) -> PathBuf {
    args.file
        .clone()
        .or_else(|| std::env::var_os("TASKS_LEDGER").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("tasks.yaml"))
}

/// The ledger, refusing to hand back one whose values are wrong. This is what
/// the verbs use: none of them should edit a file they cannot vouch for.
pub fn load(path: &Path) -> Result<Ledger> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    if let Some(message) = stale_schema_message(&raw, path) {
        bail!("{message}");
    }
    let ledger: Ledger =
        serde_yml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    let complaints = value_errors(&ledger);
    ensure!(
        complaints.is_empty(),
        "validate {}: {}",
        path.display(),
        complaints.join("; ")
    );
    Ok(ledger)
}

/// Schema 3 is rewritten by `qctl fmt`; any other version is not this binary's.
#[must_use]
pub fn stale_schema_message(raw: &str, path: &Path) -> Option<String> {
    let version = serde_yml::from_str::<serde_yml::Value>(raw)
        .ok()
        .and_then(|value| value.get("schema_version")?.as_u64())?;
    if version == u64::from(VERSION) {
        return None;
    }
    if version == 3 {
        return Some(format!(
            "{}: schema_version 3; run qctl fmt to rewrite notes into a list and set schema_version 4",
            path.display()
        ));
    }
    Some(format!(
        "{}: schema_version {version} must be {VERSION}",
        path.display()
    ))
}

/// The ledger as written, whether or not its values pass.
///
/// `check` reports every problem in one run, so it needs the parsed rows even
/// when a value is wrong — a hard failure here would throw away the schema and
/// graph findings it already has.
pub fn read(path: &Path) -> Result<Ledger> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_yml::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

/// What the contract says is wrong with these values, field by field.
#[must_use]
pub fn value_errors(ledger: &Ledger) -> Vec<String> {
    match ledger.validate() {
        Ok(()) => Vec::new(),
        Err(report) => report
            .iter()
            .map(|(field, error)| format!("{field}: {error}"))
            .collect(),
    }
}

pub fn load_value(path: &Path) -> Result<Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let yaml: serde_yml::Value =
        serde_yml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    serde_json::to_value(yaml).context("convert ledger yaml to json")
}

pub fn schema_value() -> Result<Value> {
    serde_json::from_str(CANONICAL_SCHEMA).context("embedded tasks.schema.json")
}

pub fn status(args: &LedgerArgs) -> Result<Report> {
    let path = resolve_path(args);
    let ledger = load(&path)?;
    Ok(Report::Status {
        path: path.display().to_string(),
        ledger,
    })
}

pub fn show(args: &crate::cli::IdArgs) -> Result<Report> {
    let path = resolve_path(&args.ledger);
    let ledger = load(&path)?;
    let task = if let Some(task) = ledger.queue.into_iter().find(|task| task.id == args.id) {
        Task::Queued { task }
    } else if let Some(task) = ledger.archive.into_iter().find(|task| task.id == args.id) {
        Task::Archived { task }
    } else if let Some(task) = ledger.horizon.into_iter().find(|task| task.id == args.id) {
        Task::Horizon { task }
    } else {
        bail!("no task {}", args.id);
    };
    Ok(Report::Show {
        path: path.display().to_string(),
        task,
    })
}

#[must_use]
pub fn graph_errors(ledger: &Ledger, root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let prefix = &ledger.prefix;

    for task in &ledger.archive {
        if !crate::schema::is_a_real_instant(&task.completed) {
            errors.push(format!(
                "{}: completed {} is not a moment that exists",
                task.id, task.completed
            ));
        }
    }

    for id in ledger
        .queue
        .iter()
        .map(|task| task.id.as_str())
        .chain(ledger.archive.iter().map(|task| task.id.as_str()))
        .chain(ledger.horizon.iter().map(|task| task.id.as_str()))
    {
        if !id_matches_prefix(id, prefix) {
            errors.push(format!("{id} does not match {prefix}-NNN"));
        }
    }
    errors.extend(corpus::errors(ledger));

    if let Some(active) = &ledger.active {
        if ledger.horizon.iter().any(|task| task.id == *active) {
            errors.push(format!("active {active} is on the horizon, not the queue"));
        }
        match ledger.queue.first() {
            Some(head) if head.id == *active => {
                if !head.blocked_by.is_empty() {
                    errors.push(format!("active {active} is blocked"));
                }
            }
            Some(head) => errors.push(format!(
                "active {active} must be queue[0] (found {})",
                head.id
            )),
            None => errors.push(format!("active {active} is not in the queue")),
        }
    }

    let queue_index: HashMap<&str, usize> = ledger
        .queue
        .iter()
        .enumerate()
        .map(|(index, task)| (task.id.as_str(), index))
        .collect();
    for (index, task) in ledger.queue.iter().enumerate() {
        for blocker in &task.blocked_by {
            match queue_index.get(blocker.as_str()) {
                Some(blocker_index) if *blocker_index < index => {}
                Some(_) => errors.push(format!("{} <- {blocker} is not earlier", task.id)),
                None => errors.push(format!("{} <- {blocker} is not queued", task.id)),
            }
        }
    }

    if ledger.style.archive_order == ArchiveOrder::NewestFirst {
        let stamps: Vec<&str> = ledger
            .archive
            .iter()
            .map(|task| task.completed.as_str())
            .collect();
        let mut sorted = stamps.clone();
        sorted.sort_unstable();
        sorted.reverse();
        if stamps != sorted {
            errors.push("archive is not newest-first by completed".into());
        }
    }

    let parent = root.parent().unwrap_or(Path::new("."));
    for plan in ledger
        .queue
        .iter()
        .filter_map(|task| task.plan.as_deref())
        .chain(
            ledger
                .archive
                .iter()
                .filter_map(|task| task.plan.as_deref()),
        )
        .chain(
            ledger
                .horizon
                .iter()
                .filter_map(|task| task.plan.as_deref()),
        )
    {
        if !parent.join(plan).is_file() {
            errors.push(format!("missing plan {plan}"));
        }
    }

    errors
}

fn id_matches_prefix(id: &str, prefix: &str) -> bool {
    let Some(number) = id
        .strip_prefix(prefix)
        .and_then(|rest| rest.strip_prefix('-'))
    else {
        return false;
    };
    number.len() >= 3 && number.bytes().all(|byte| byte.is_ascii_digit())
}

pub fn next_id(ledger: &Ledger) -> Result<String> {
    let next = corpus::next_number(ledger);
    ensure!(next <= 999_999, "id space exhausted");
    Ok(format!("{}-{next:03}", ledger.prefix))
}

#[cfg(test)]
mod tests {
    use super::id_matches_prefix;

    #[test]
    fn accepts_prefixed_ids() {
        assert!(id_matches_prefix("OMX-001", "OMX"));
        assert!(id_matches_prefix("PST-037", "PST"));
        assert!(!id_matches_prefix("OMX-01", "OMX"));
        assert!(!id_matches_prefix("KAI-001", "OMX"));
    }
}
