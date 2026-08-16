use crate::cli::{AddArgs, ArchiveArgs, InitArgs};
use crate::ledger::{load, next_id, resolve_path};
use anyhow::{Context, Result, bail, ensure};
use serde_yml::{Mapping, Value};
use std::fs;
use std::path::Path;
use time::OffsetDateTime;
use time::macros::format_description;

pub fn init(args: &InitArgs) -> Result<()> {
    let prefix = args.prefix.to_ascii_uppercase();
    ensure!(
        valid_prefix(&prefix),
        "prefix must match [A-Z][A-Z0-9]{{1,7}}"
    );
    let path = resolve_path(&args.ledger);
    if path.exists() && !args.force {
        bail!("{} already exists (pass --force)", path.display());
    }
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }
    let body = format!(
        "# yaml-language-server: $schema=https://raw.githubusercontent.com/victor-software-house/qctl/main/schema/tasks.schema.json\nschema_version: 1\nprefix: {prefix}\nactive: null\nqueue: []\narchive: []\nhorizon: []\n"
    );
    fs::write(&path, body).with_context(|| path.display().to_string())?;
    println!("wrote {}", path.display());
    Ok(())
}

pub fn add(args: &AddArgs) -> Result<()> {
    let path = resolve_path(&args.ledger);
    let ledger = load(&path)?;
    let id = next_id(&ledger)?;
    let mut value = read_yaml(&path)?;
    let mapping = root_map(&mut value)?;
    let queue = sequence(mapping, "queue")?;
    let mut item = Mapping::new();
    item.insert(Value::from("id"), Value::from(id.clone()));
    item.insert(Value::from("title"), Value::from(args.title.clone()));
    item.insert(Value::from("scope"), Value::from(args.scope.clone()));
    item.insert(Value::from("outcome"), Value::from(args.outcome.clone()));
    item.insert(Value::from("blocked_by"), Value::Sequence(Vec::new()));
    item.insert(
        Value::from("acceptance"),
        Value::from(args.acceptance.clone()),
    );
    if let Some(patch) = &args.patch {
        item.insert(Value::from("patch"), Value::from(patch.clone()));
    }
    queue.push(Value::Mapping(item));
    write_yaml(&path, &value)?;
    println!("{id}");
    Ok(())
}

pub fn start(args: &crate::cli::IdArgs) -> Result<()> {
    let path = resolve_path(&args.ledger);
    let ledger = load(&path)?;
    let index = ledger
        .queue
        .iter()
        .position(|task| task.id == args.id)
        .with_context(|| format!("{} is not queued", args.id))?;
    ensure!(
        ledger.queue[index].blocked_by.is_empty(),
        "{} is blocked",
        args.id
    );
    let mut value = read_yaml(&path)?;
    let mapping = root_map(&mut value)?;
    let queue = sequence(mapping, "queue")?;
    let item = queue.remove(index);
    queue.insert(0, item);
    mapping.insert(Value::from("active"), Value::from(args.id.clone()));
    write_yaml(&path, &value)?;
    println!("active  {}", args.id);
    Ok(())
}

pub fn archive(args: &ArchiveArgs) -> Result<()> {
    let path = resolve_path(&args.ledger);
    let today = OffsetDateTime::now_utc()
        .date()
        .format(format_description!("[year]-[month]-[day]"))
        .context("format date")?;
    let mut value = read_yaml(&path)?;
    let mapping = root_map(&mut value)?;
    let queue = sequence(mapping, "queue")?;
    let position = queue
        .iter()
        .position(|item| item.get("id").and_then(Value::as_str) == Some(args.id.as_str()))
        .with_context(|| format!("{} is not queued", args.id))?;
    let mut item = queue.remove(position);
    if let Some(map) = item.as_mapping_mut() {
        map.remove(Value::from("blocked_by"));
        map.remove(Value::from("acceptance"));
        map.insert(Value::from("completed"), Value::from(today));
        map.insert(Value::from("evidence"), Value::from(args.evidence.clone()));
        map.insert(
            Value::from("disposition"),
            Value::from(args.disposition.as_str()),
        );
    }
    let archive = sequence(mapping, "archive")?;
    archive.insert(0, item);
    let next = mapping
        .get("queue")
        .and_then(Value::as_sequence)
        .and_then(|queue| queue.first())
        .and_then(|item| item.get("id"))
        .cloned();
    mapping.insert(Value::from("active"), next.unwrap_or(Value::Null));
    write_yaml(&path, &value)?;
    println!("archived  {}", args.id);
    Ok(())
}

fn read_yaml(path: &Path) -> Result<Value> {
    let raw = fs::read_to_string(path).with_context(|| path.display().to_string())?;
    serde_yml::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn write_yaml(path: &Path, value: &Value) -> Result<()> {
    fs::write(path, serde_yml::to_string(value)?).with_context(|| path.display().to_string())
}

fn root_map(value: &mut Value) -> Result<&mut Mapping> {
    value
        .as_mapping_mut()
        .context("ledger root must be a mapping")
}

fn sequence<'a>(mapping: &'a mut Mapping, key: &str) -> Result<&'a mut Vec<Value>> {
    mapping
        .get_mut(Value::from(key))
        .and_then(Value::as_sequence_mut)
        .with_context(|| format!("{key} must be a sequence"))
}

fn valid_prefix(prefix: &str) -> bool {
    let mut chars = prefix.chars();
    matches!(chars.next(), Some('A'..='Z'))
        && prefix.len() >= 2
        && prefix.len() <= 8
        && chars.all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
}
