//! Reading a ledger: where it is, whether its values pass, and the graph
//! rules a JSON Schema cannot state.

mod corpus;
pub(crate) mod order;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};
use ctl_core::input::Input;
use serde::Deserialize;
use serde_json::Value;

use crate::cli::LedgerArgs;
use crate::report::{Report, Task};
use crate::schema::ArchiveOrder;
pub use crate::schema::{Ledger, VERSION};

/// The schema qctl was built with, generated from [`crate::schema`].
pub const CANONICAL_SCHEMA: &str = include_str!("../schema/tasks.schema.json");

/// The ledger file: `--file` or `QCTL_FILE`, else `tasks.yaml`.
#[must_use]
pub fn resolve_path(args: &LedgerArgs) -> PathBuf {
    args.file
        .clone()
        .unwrap_or_else(|| PathBuf::from("tasks.yaml"))
}

/// The ledger, refusing to hand back one whose values are wrong. This is what
/// the verbs use: none of them should edit a file they cannot vouch for.
///
/// # Errors
///
/// Fails when the file cannot be read or is not YAML, when it declares a
/// schema version other than [`VERSION`], or when a value breaks a schema
/// rule. Each message names the file and the line.
pub fn load(path: &Path) -> Result<Ledger> {
    let input = Input::read(path)?;
    if let Some(message) = stale_schema_message(&input) {
        bail!("{message}");
    }
    checked(&input)
}

/// A ledger parsed and validated through ctl-core's declared input, so every
/// complaint names the file and the line it is on.
///
/// # Errors
///
/// Fails when the text does not parse as a ledger or a value breaks a schema
/// rule.
pub fn checked(input: &Input) -> Result<Ledger> {
    let ledger: Ledger = input.parse()?;
    input.check(&ledger, &())?;
    Ok(ledger)
}

/// The version a file declares, read before its rows so a file written for
/// another schema is named as such rather than refused field by field.
#[must_use]
pub fn schema_version(input: &Input) -> Option<u64> {
    #[derive(Deserialize)]
    struct Declared {
        schema_version: Option<u64>,
    }
    input.parse::<Declared>().ok()?.schema_version
}

/// Schema 3 is rewritten by `qctl fmt`; any other version is not this binary's.
#[must_use]
pub fn stale_schema_message(input: &Input) -> Option<String> {
    let version = schema_version(input)?;
    let path = input.name();
    if version == u64::from(VERSION) {
        return None;
    }
    if version == 3 {
        return Some(format!(
            "{path}: schema_version 3; run qctl fmt to rewrite notes into a list and set schema_version 4"
        ));
    }
    Some(format!(
        "{path}: schema_version {version} must be {VERSION}"
    ))
}

/// The ledger as written, whether or not its values pass.
///
/// `check` reports every problem in one run, so it needs the parsed rows even
/// when a value is wrong — a hard failure here would throw away the schema and
/// graph findings it already has.
///
/// # Errors
///
/// Fails when the file cannot be read or does not parse as a ledger.
pub fn read(path: &Path) -> Result<Ledger> {
    Ok(Input::read(path)?.parse()?)
}

/// The ledger as JSON, for the JSON Schema check.
///
/// # Errors
///
/// Fails when the text is not YAML.
pub fn load_value(input: &Input) -> Result<Value> {
    Ok(input.parse()?)
}

/// [`CANONICAL_SCHEMA`] as JSON.
///
/// # Errors
///
/// Fails when the embedded schema is not JSON, which the generated file
/// never is.
pub fn schema_value() -> Result<Value> {
    serde_json::from_str(CANONICAL_SCHEMA).context("embedded tasks.schema.json")
}

/// `qctl status`: the whole ledger, for the active row and the queue.
///
/// # Errors
///
/// Fails when the ledger does not [`load`].
pub fn status(args: &LedgerArgs) -> Result<Report> {
    let path = resolve_path(args);
    let ledger = load(&path)?;
    Ok(Report::Status {
        path: path.display().to_string(),
        ledger,
    })
}

/// `qctl show`: one row, from whichever list holds it.
///
/// # Errors
///
/// Fails when the ledger does not [`load`] or no list holds the id.
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

/// Every rule the ledger breaks that a JSON Schema cannot state: real
/// completion stamps, id prefixes and the id corpus, the active row, blocker
/// order, archive order, and plan files. `root` is the ledger's path; plans
/// resolve next to it.
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
    (3..=6).contains(&number.len()) && number.bytes().all(|byte| byte.is_ascii_digit())
}

/// The id after the highest one any list holds.
///
/// # Errors
///
/// Fails when the highest number is already the largest an id can carry,
/// `999999`.
pub fn next_id(ledger: &Ledger) -> Result<String> {
    let highest = corpus::highest_number(ledger);
    ensure!(highest < corpus::MAX_TASK_NUMBER, "id space exhausted");
    Ok(format!("{}-{:03}", ledger.prefix, highest + 1))
}

#[cfg(test)]
mod tests {
    use super::id_matches_prefix;

    #[test]
    fn accepts_prefixed_ids() {
        assert!(id_matches_prefix("FRK-001", "FRK"));
        assert!(id_matches_prefix("VER-037", "VER"));
        assert!(!id_matches_prefix("FRK-01", "FRK"));
        assert!(!id_matches_prefix("CTC-001", "FRK"));
    }
}
