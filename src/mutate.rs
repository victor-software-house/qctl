use crate::cli::{AddArgs, ArchiveArgs, InitArgs};
use crate::document::Document;
use crate::ledger::{load, next_id, resolve_path};
use crate::schema::QueuedTask;
use anyhow::{Context, Result, bail, ensure};
use std::fs;
use std::path::Path;
use time::macros::format_description;
use time::{OffsetDateTime, UtcOffset};

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
    let version = crate::schema::VERSION;
    let body = format!(
        "# yaml-language-server: $schema=https://raw.githubusercontent.com/victor-software-house/qctl/main/schema/tasks.schema.json\nschema_version: {version}\nprefix: {prefix}\nactive: null\nqueue: []\narchive: []\nhorizon: []\n"
    );
    fs::write(&path, body).with_context(|| path.display().to_string())?;
    println!("wrote {}", path.display());
    Ok(())
}

pub fn add(args: &AddArgs) -> Result<()> {
    let path = resolve_path(&args.ledger);
    let ledger = load(&path)?;
    let id = next_id(&ledger)?;
    let row = QueuedTask {
        id: id.clone(),
        title: args.title.clone(),
        scope: args.scope.clone(),
        outcome: args.outcome.clone(),
        blocked_by: Vec::new(),
        acceptance: args.acceptance.clone(),
        patch: args.patch.clone(),
        plan: None,
        links: Vec::new(),
        notes: None,
    };

    let mut document = read(&path)?;
    document.append("queue", &yaml_serde::to_value(&row)?)?;
    write(&path, document)?;
    println!("{id}");
    Ok(())
}

pub fn start(args: &crate::cli::IdArgs) -> Result<()> {
    let path = resolve_path(&args.ledger);
    let ledger = load(&path)?;
    let task = ledger
        .queue
        .iter()
        .find(|task| task.id == args.id)
        .with_context(|| format!("{} is not queued", args.id))?;
    ensure!(task.blocked_by.is_empty(), "{} is blocked", args.id);

    let mut document = read(&path)?;
    document.move_to_front("queue", &args.id)?;
    document.set("active", yaml_serde::Value::from(args.id.as_str()))?;
    write(&path, document)?;
    println!("active  {}", args.id);
    Ok(())
}

pub fn archive(args: &ArchiveArgs) -> Result<()> {
    let path = resolve_path(&args.ledger);
    let ledger = load(&path)?;
    ensure!(
        ledger.queue.iter().any(|task| task.id == args.id),
        "{} is not queued",
        args.id
    );
    let now = now_in(&ledger.style.timezone)?;

    let mut document = read(&path)?;
    // The row loses what only a queued row carries and gains what only an
    // archived one does on the way across, while it stands alone.
    document.move_between(
        "queue",
        "archive",
        &args.id,
        &["blocked_by", "acceptance"],
        &[
            ("completed", yaml_serde::Value::from(now.as_str())),
            ("evidence", yaml_serde::to_value(&args.evidence)?),
            (
                "disposition",
                yaml_serde::Value::from(args.disposition.as_str()),
            ),
        ],
    )?;
    // A blocker is only resolved against the queue, so a row still naming this
    // id would fail `check` the moment this verb returns. What each row keeps
    // comes from the ledger this verb already validated, not from reading its
    // own output back.
    for task in &ledger.queue {
        if !task.blocked_by.contains(&args.id) {
            continue;
        }
        let kept: Vec<String> = task
            .blocked_by
            .iter()
            .filter(|blocker| **blocker != args.id)
            .cloned()
            .collect();
        document.rewrite_blockers("queue", &task.id, &kept)?;
    }
    let next = document.ids("queue")?.first().cloned();
    document.set(
        "active",
        next.map_or(yaml_serde::Value::Null, |id| {
            yaml_serde::Value::from(id.as_str())
        }),
    )?;
    write(&path, document)?;
    println!("archived  {}", args.id);
    Ok(())
}

/// Now, in the zone this ledger declares, written the one way a stamp is
/// written. No offset: the file says which zone its stamps are in, once.
fn now_in(zone: &str) -> Result<String> {
    let offset = UtcOffset::parse(
        zone,
        format_description!("[offset_hour sign:mandatory]:[offset_minute]"),
    )
    .with_context(|| format!("{zone} is not an offset from UTC"))?;
    OffsetDateTime::now_utc()
        .to_offset(offset)
        .format(format_description!(
            "[year]-[month]-[day]T[hour]:[minute]:[second]"
        ))
        .context("format the moment this row left the queue")
}

fn read(path: &Path) -> Result<Document> {
    let source = fs::read_to_string(path).with_context(|| path.display().to_string())?;
    Ok(Document::new(source))
}

/// Write only a document that still reads as the ledger it was, so a verb can
/// never leave a file behind that the next one cannot open.
///
/// A ledger whose style asks for it is normalized on the way out. Off by
/// default, because normalizing touches lines the verb had no business in, and a
/// diff that shows only the work is worth more than one that is always tidy.
fn write(path: &Path, document: Document) -> Result<()> {
    let source = document.into_source();
    crate::document::must_still_parse(&source)?;
    let edited: crate::ledger::Ledger =
        serde_yml::from_str(&source).context("read back what this verb wrote")?;
    let source = if edited.style.normalize_on_write {
        crate::format::normalized(&source, &edited)?
    } else {
        source
    };
    fs::write(path, source).with_context(|| path.display().to_string())
}

fn valid_prefix(prefix: &str) -> bool {
    let mut chars = prefix.chars();
    matches!(chars.next(), Some('A'..='Z'))
        && prefix.len() >= 2
        && prefix.len() <= 8
        && chars.all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
}
