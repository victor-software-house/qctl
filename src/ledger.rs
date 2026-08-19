use crate::cli::LedgerArgs;
use anyhow::{Context, Result, bail, ensure};
use garde::Validate;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use time::macros::{format_description, offset};

pub use crate::schema::Ledger;

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
    let ledger = read(path)?;
    let complaints = value_errors(&ledger);
    ensure!(
        complaints.is_empty(),
        "validate {}: {}",
        path.display(),
        complaints.join("; ")
    );
    Ok(ledger)
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

pub fn print_status(args: &LedgerArgs) -> Result<()> {
    let path = resolve_path(args);
    let ledger = load(&path)?;
    match ledger.active.as_deref() {
        Some(id) => println!("active  {id}"),
        None => println!("active  (none)"),
    }
    if ledger.queue.is_empty() {
        println!("queue   (empty)");
    } else {
        for (index, task) in ledger.queue.iter().enumerate() {
            let mark = if ledger.active.as_deref() == Some(task.id.as_str()) {
                "*"
            } else {
                " "
            };
            println!("queue{mark} {:>2}  {}  {}", index + 1, task.id, task.title);
        }
    }
    if ledger.horizon.is_empty() {
        return Ok(());
    }
    println!("horizon {}", ledger.horizon.len());
    for task in &ledger.horizon {
        println!("        {}  [{}]  {}", task.id, task.kind, task.title);
    }
    Ok(())
}

/// A stamp as it reads where the work happened. The ledger stores UTC so that
/// stamps sort wherever they are read; a person wants the hour they were at the
/// desk. Brazil has not observed daylight saving since 2019, so that is a fixed
/// three hours — deliberately fixed, not the reader's local zone, so the same
/// ledger reads the same everywhere. The offset is printed with it, because a
/// bare wall-clock time copied out of a terminal says nothing. An unparseable
/// stamp is printed as written, because `show` has to keep working on a row
/// somebody hand-edited wrong.
fn where_the_work_happens(stamp: &str) -> String {
    let shown = format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]:[offset_minute]"
    );
    match OffsetDateTime::parse(stamp, &Rfc3339) {
        Ok(at) => at
            .to_offset(offset!(-3))
            .format(shown)
            .unwrap_or_else(|_| stamp.to_owned()),
        Err(_) => stamp.to_owned(),
    }
}

pub fn print_show(args: &crate::cli::IdArgs) -> Result<()> {
    let path = resolve_path(&args.ledger);
    let ledger = load(&path)?;
    if let Some(task) = ledger.queue.iter().find(|task| task.id == args.id) {
        println!("{}  {}", task.id, task.title);
        println!("scope     {}", task.scope);
        println!("outcome   {}", task.outcome);
        if let Some(patch) = &task.patch {
            println!("patch     {patch}");
        }
        return Ok(());
    }
    if let Some(task) = ledger.archive.iter().find(|task| task.id == args.id) {
        println!(
            "{}  {}  (archived {})",
            task.id,
            task.title,
            where_the_work_happens(&task.completed)
        );
        if let Some(notes) = &task.notes {
            println!("notes     {notes}");
        }
        return Ok(());
    }
    if let Some(task) = ledger.horizon.iter().find(|task| task.id == args.id) {
        println!("{}  {}  (horizon {})", task.id, task.title, task.kind);
        println!("scope     {}", task.scope);
        println!("outcome   {}", task.outcome);
        println!("open      {}", task.open);
        return Ok(());
    }
    bail!("no task {}", args.id);
}

#[must_use]
pub fn graph_errors(ledger: &Ledger, root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let prefix = &ledger.prefix;

    let mut seen = HashSet::new();
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
        if !seen.insert(id) {
            errors.push(format!("duplicate id {id}"));
        }
    }

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

    let dates: Vec<&str> = ledger
        .archive
        .iter()
        .map(|task| task.completed.as_str())
        .collect();
    let mut sorted = dates.clone();
    sorted.sort_unstable();
    sorted.reverse();
    if dates != sorted {
        errors.push("archive is not newest-first by completed date".into());
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
    let mut max = 0_u32;
    for id in ledger
        .queue
        .iter()
        .map(|task| task.id.as_str())
        .chain(ledger.archive.iter().map(|task| task.id.as_str()))
        .chain(ledger.horizon.iter().map(|task| task.id.as_str()))
    {
        let Some((_, number)) = id.rsplit_once('-') else {
            continue;
        };
        if let Ok(value) = number.parse::<u32>() {
            max = max.max(value);
        }
    }
    ensure!(max < 999_999, "id space exhausted");
    Ok(format!("{}-{:03}", ledger.prefix, max + 1))
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
