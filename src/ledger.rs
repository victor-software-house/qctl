use crate::cli::LedgerArgs;
use anyhow::{Context, Result, bail, ensure};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const CANONICAL_SCHEMA: &str = include_str!("../schema/tasks.schema.json");

#[derive(Debug, Deserialize)]
pub struct Ledger {
    pub schema_version: u32,
    pub prefix: String,
    pub active: Option<String>,
    pub queue: Vec<QueuedTask>,
    pub archive: Vec<ArchivedTask>,
    #[serde(default)]
    pub horizon: Vec<HorizonTask>,
}

#[derive(Debug, Deserialize)]
pub struct QueuedTask {
    pub id: String,
    pub title: String,
    pub scope: String,
    pub outcome: String,
    pub blocked_by: Vec<String>,
    #[allow(dead_code)]
    pub acceptance: Vec<String>,
    pub patch: Option<String>,
    pub plan: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ArchivedTask {
    pub id: String,
    pub title: String,
    pub completed: String,
    pub plan: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct HorizonTask {
    pub id: String,
    pub title: String,
    pub scope: String,
    pub outcome: String,
    pub kind: String,
    pub open: String,
    pub plan: Option<String>,
}

#[must_use]
pub fn resolve_path(args: &LedgerArgs) -> PathBuf {
    args.file
        .clone()
        .or_else(|| std::env::var_os("TASKS_LEDGER").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("tasks.yaml"))
}

pub fn load(path: &Path) -> Result<Ledger> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_yml::from_str(&raw).with_context(|| format!("parse {}", path.display()))
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
        println!("{}  {}  (archived {})", task.id, task.title, task.completed);
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

    if ledger.schema_version != 1 {
        errors.push(format!(
            "schema_version must be 1, got {}",
            ledger.schema_version
        ));
    }

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
