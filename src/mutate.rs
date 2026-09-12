use crate::cli::{
    AddArgs, ArchiveArgs, CloseFromGitArgs, Disposition, InitArgs, ParkArgs, PromoteArgs,
};
use crate::document::Document;
use crate::ledger::order::{self, Row};
use crate::ledger::{load, next_id, resolve_path};
use crate::report::{Destination, Report};
use crate::schema::{HorizonTask, QueuedTask};
use crate::trailers;
use anyhow::{Context, Result, bail, ensure};
use indoc::formatdoc;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use time::macros::format_description;
use time::{OffsetDateTime, UtcOffset};

pub fn init(args: &InitArgs) -> Result<Report> {
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
    let body = formatdoc! {"
        # yaml-language-server: $schema=https://raw.githubusercontent.com/victor-software-house/qctl/main/schema/tasks.schema.json
        schema_version: {version}
        prefix: {prefix}
        active: null
        queue: []
        archive: []
        horizon: []
    "};
    fs::write(&path, body).with_context(|| path.display().to_string())?;
    Ok(Report::Initialized {
        path: path.display().to_string(),
        prefix,
    })
}

pub fn add(args: &AddArgs) -> Result<Report> {
    if args.horizon {
        ensure!(
            args.blocked_by.is_empty(),
            "horizon rows have no blocked_by"
        );
        ensure!(
            args.acceptance.is_empty(),
            "horizon rows have no acceptance"
        );
        add_horizon(args)
    } else {
        ensure!(
            args.kind.is_none() && args.open.is_none(),
            "--kind and --open belong to --horizon"
        );
        add_queue(args)
    }
}

fn add_queue(args: &AddArgs) -> Result<Report> {
    ensure!(
        !args.acceptance.is_empty(),
        "add to the queue needs --acceptance"
    );
    let path = resolve_path(&args.ledger);
    let ledger = load(&path)?;
    require_plan(&path, args.plan.as_deref())?;
    let id = next_id(&ledger)?;
    let row = QueuedTask {
        id: id.clone(),
        title: args.title.clone(),
        scope: args.scope.clone(),
        outcome: args.outcome.clone(),
        blocked_by: args.blocked_by.clone(),
        acceptance: args.acceptance.clone(),
        patch: args.patch.clone(),
        plan: args.plan.clone(),
        links: args.links.clone(),
        notes: args.note.clone(),
    };

    let mut ids: Vec<String> = ledger.queue.iter().map(|task| task.id.clone()).collect();
    let insertion = insertion_index(&ids, args.before.as_deref(), args.after.as_deref())?;
    if insertion == 0
        && !ids.is_empty()
        && let Some(active) = ledger.active.as_deref()
    {
        bail!(
            "add --before {before} would make {id} queue[0] while active is {active}",
            before = args.before.as_deref().unwrap_or(active)
        );
    }
    let mut ordered: Vec<Row<'_>> = ledger
        .queue
        .iter()
        .map(|task| Row {
            id: &task.id,
            blockers: &task.blocked_by,
        })
        .collect();
    ordered.insert(
        insertion,
        Row {
            id: &id,
            blockers: &args.blocked_by,
        },
    );
    order::validate(&ordered)?;

    let mut document = read(&path)?;
    document.append("queue", &yaml_serde::to_value(&row)?)?;
    if insertion < ids.len() {
        ids.insert(insertion, id.clone());
        document.reorder_rows("queue", &ids)?;
    }
    write(&path, document)?;
    Ok(Report::Added {
        path: path.display().to_string(),
        id,
        destination: Destination::Queue,
    })
}

fn add_horizon(args: &AddArgs) -> Result<Report> {
    let kind = args.kind.context("--horizon needs --kind")?;
    let open = args.open.as_deref().context("--horizon needs --open")?;
    let path = resolve_path(&args.ledger);
    let ledger = load(&path)?;
    require_plan(&path, args.plan.as_deref())?;
    let id = next_id(&ledger)?;
    let row = HorizonTask {
        id: id.clone(),
        title: args.title.clone(),
        scope: args.scope.clone(),
        outcome: args.outcome.clone(),
        kind,
        open: open.to_owned(),
        patch: args.patch.clone(),
        plan: args.plan.clone(),
        links: args.links.clone(),
        notes: args.note.clone(),
    };
    let mut document = read(&path)?;
    document.append("horizon", &yaml_serde::to_value(&row)?)?;
    write(&path, document)?;
    Ok(Report::Added {
        path: path.display().to_string(),
        id,
        destination: Destination::Horizon,
    })
}

fn insertion_index(ids: &[String], before: Option<&str>, after: Option<&str>) -> Result<usize> {
    match (before, after) {
        (None, None) => Ok(ids.len()),
        (Some(_), Some(_)) => bail!("use --before or --after, not both"),
        (Some(before), None) => ids
            .iter()
            .position(|id| id == before)
            .with_context(|| format!("{before} is not queued")),
        (None, Some(after)) => {
            let index = ids
                .iter()
                .position(|id| id == after)
                .with_context(|| format!("{after} is not queued"))?;
            Ok(index + 1)
        }
    }
}

pub fn start(args: &crate::cli::IdArgs) -> Result<Report> {
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
    Ok(Report::Started {
        path: path.display().to_string(),
        id: args.id.clone(),
    })
}

pub fn park(args: &ParkArgs) -> Result<Report> {
    let path = resolve_path(&args.ledger);
    let ledger = load(&path)?;
    ensure!(
        ledger.queue.iter().any(|task| task.id == args.id),
        "{} is not queued",
        args.id
    );
    for task in &ledger.queue {
        ensure!(
            !task.blocked_by.contains(&args.id),
            "{} still blocks {}; a parked row remains a real blocker",
            args.id,
            task.id
        );
    }

    let mut document = read(&path)?;
    document.move_to_end(
        "queue",
        "horizon",
        &args.id,
        &["blocked_by", "acceptance"],
        &[
            ("kind", yaml_serde::Value::from(args.kind.to_string())),
            ("open", yaml_serde::Value::from(args.open.as_str())),
        ],
    )?;
    if ledger.active.as_deref() == Some(args.id.as_str()) {
        document.set("active", yaml_serde::Value::Null)?;
    }
    write(&path, document)?;
    Ok(Report::Parked {
        path: path.display().to_string(),
        id: args.id.clone(),
    })
}

pub fn promote(args: &PromoteArgs) -> Result<Report> {
    ensure!(
        !args.acceptance.is_empty(),
        "promote onto the queue needs --acceptance"
    );
    let path = resolve_path(&args.ledger);
    let ledger = load(&path)?;
    ensure!(
        ledger.horizon.iter().any(|task| task.id == args.id),
        "{} is not on the horizon",
        args.id
    );
    for blocker in &args.blocked_by {
        ensure!(
            ledger.queue.iter().any(|queued| queued.id == *blocker),
            "{} <- {blocker} is not queued",
            args.id
        );
    }

    let mut document = read(&path)?;
    document.move_to_end(
        "horizon",
        "queue",
        &args.id,
        &["kind", "open"],
        &[
            ("blocked_by", yaml_serde::to_value(&args.blocked_by)?),
            ("acceptance", yaml_serde::to_value(&args.acceptance)?),
        ],
    )?;
    write(&path, document)?;
    Ok(Report::Promoted {
        path: path.display().to_string(),
        id: args.id.clone(),
    })
}

pub fn archive(args: &ArchiveArgs) -> Result<Report> {
    let path = archive_row(args)?;
    Ok(Report::Archived {
        path,
        id: args.id.clone(),
    })
}

fn archive_row(args: &ArchiveArgs) -> Result<String> {
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
    Ok(path.display().to_string())
}

pub fn close_from_git(args: &CloseFromGitArgs) -> Result<Report> {
    let path = resolve_path(&args.ledger);
    let root = path.parent().unwrap_or(&path);
    let closed = if args.pre_push {
        let stdin = std::io::read_to_string(std::io::stdin()).context("read pre-push stdin")?;
        trailers::closed_ids_pre_push(root, &stdin)?
    } else if let Some(range) = &args.range {
        trailers::closed_ids_rev(root, &[range.as_str()])?
    } else {
        trailers::closed_ids_rev(root, &[])?
    };
    let ledger = load(&path)?;
    let queued: HashSet<String> = ledger.queue.iter().map(|task| task.id.clone()).collect();
    let mut seen = HashSet::new();
    let mut archived = Vec::new();
    for (id, sha) in closed {
        if !queued.contains(&id) || !seen.insert(id.clone()) {
            continue;
        }
        archive_row(&ArchiveArgs {
            id: id.clone(),
            ledger: args.ledger.clone(),
            evidence: vec![sha],
            disposition: Disposition::Completed,
        })?;
        archived.push(id);
    }
    let ledger = load(&path)?;
    if let Some(active) = &ledger.active
        && seen.contains(active)
    {
        bail!("active {active} still names a closed id");
    }
    Ok(Report::ClosedFromGit {
        path: path.display().to_string(),
        retry_push: args.pre_push && !archived.is_empty(),
        archived,
    })
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

pub(crate) fn read(path: &Path) -> Result<Document> {
    let source = fs::read_to_string(path).with_context(|| path.display().to_string())?;
    Ok(Document::new(source))
}

/// Write only a document that still reads as the ledger it was, so a verb can
/// never leave a file behind that the next one cannot open.
///
/// A ledger whose style asks for it is normalized on the way out. Off by
/// default, because normalizing touches lines the verb had no business in, and a
/// diff that shows only the work is worth more than one that is always tidy.
pub(crate) fn write(path: &Path, document: Document) -> Result<()> {
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

pub(crate) fn require_plan(ledger: &Path, plan: Option<&str>) -> Result<()> {
    let Some(plan) = plan else {
        return Ok(());
    };
    ensure!(!plan.is_empty(), "--plan needs a path");
    crate::schema::inside_the_repo(plan, &())
        .map_err(|error| anyhow::anyhow!("plan {plan} {error}"))?;
    let parent = ledger.parent().unwrap_or(Path::new("."));
    ensure!(parent.join(plan).is_file(), "missing plan {plan}");
    Ok(())
}

fn valid_prefix(prefix: &str) -> bool {
    let mut chars = prefix.chars();
    matches!(chars.next(), Some('A'..='Z'))
        && prefix.len() >= 2
        && prefix.len() <= 8
        && chars.all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
}
