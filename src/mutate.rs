//! The verbs that write a ledger: `init`, `add`, `start`, `park`, `promote`,
//! `archive`, `close-from-git`, and `close-from-openspec`.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail, ensure};
use ctl_core::input::Input;
use indoc::formatdoc;
use time::macros::format_description;
use time::{OffsetDateTime, UtcOffset};

use crate::cli::{
    AddArgs, ArchiveArgs, CloseFromGitArgs, Disposition, InitArgs, LedgerArgs, ParkArgs,
    PromoteArgs,
};
use crate::document::{Change, Document};
use crate::ledger::order::{self, Row};
use crate::ledger::{load, next_id, resolve_path};
use crate::openspec::{self, Layout, Moved};
use crate::report::{Destination, Report};
use crate::schema::{HorizonTask, ListItems, QueuedTask, Style};
use crate::trailers;

/// `qctl init`: write an empty ledger for a prefix.
///
/// # Errors
///
/// Fails when the prefix does not match `[A-Z][A-Z0-9]{1,7}`, when the file
/// exists and `--force` is not set, or when the file or its directory cannot
/// be written.
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

/// `qctl add`: write a row under the next free id, on the queue or, with
/// `--horizon`, on the horizon.
///
/// # Errors
///
/// Fails when the flags do not fit the list, when the ledger does not
/// [`load`], when `--plan` names no file next to the ledger, or when the id
/// space is used up. A queue row also fails when `--before` or `--after`
/// names no queued id, when it would sit ahead of the active row, or when a
/// blocker would not sit earlier than it. Last, the edited file must write.
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
    one_line_notes(&ledger.style, &args.note)?;
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

    let mut document = read(&path, &ledger.style)?;
    document.append("queue", &row)?;
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
    one_line_notes(&ledger.style, &args.note)?;
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
    let mut document = read(&path, &ledger.style)?;
    document.append("horizon", &row)?;
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

/// `qctl start`: make an unblocked queued row active and move it to the
/// front of the queue.
///
/// # Errors
///
/// Fails when the ledger does not [`load`], when the id is not queued or is
/// blocked, or when the file cannot be written.
pub fn start(args: &crate::cli::IdArgs) -> Result<Report> {
    let path = resolve_path(&args.ledger);
    let ledger = load(&path)?;
    let task = ledger
        .queue
        .iter()
        .find(|task| task.id == args.id)
        .with_context(|| format!("{} is not queued", args.id))?;
    ensure!(task.blocked_by.is_empty(), "{} is blocked", args.id);

    let mut document = read(&path, &ledger.style)?;
    document.move_to_front("queue", &args.id)?;
    document.set("active", args.id.as_str())?;
    write(&path, document)?;
    Ok(Report::Started {
        path: path.display().to_string(),
        id: args.id.clone(),
    })
}

/// `qctl park`: move a queued row to the end of the horizon with its `kind`
/// and `open`, dropping `acceptance` and `blocked_by`. Parking the active row
/// clears `active`.
///
/// # Errors
///
/// Fails when the ledger does not [`load`], when the id is not queued, when
/// another queued row still names it as a blocker, or when the file cannot
/// be written.
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

    let mut document = read(&path, &ledger.style)?;
    document.move_to_end(
        "queue",
        "horizon",
        &args.id,
        &[
            Change::unset("blocked_by"),
            Change::unset("acceptance"),
            Change::set("kind", args.kind.to_string()),
            Change::set("open", args.open.as_str()),
        ],
    )?;
    if ledger.active.as_deref() == Some(args.id.as_str()) {
        document.set("active", &None::<&str>)?;
    }
    write(&path, document)?;
    Ok(Report::Parked {
        path: path.display().to_string(),
        id: args.id.clone(),
    })
}

/// `qctl promote`: move a horizon row to the end of the queue with its
/// acceptance and blockers, dropping `kind` and `open`.
///
/// # Errors
///
/// Fails when no `--acceptance` is given, when the ledger does not [`load`],
/// when the id is not on the horizon, when a blocker is not queued, or when
/// the file cannot be written.
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

    let mut document = read(&path, &ledger.style)?;
    document.move_to_end(
        "horizon",
        "queue",
        &args.id,
        &[
            Change::unset("kind"),
            Change::unset("open"),
            Change::items("blocked_by", &args.blocked_by),
            Change::items("acceptance", &args.acceptance),
        ],
    )?;
    write(&path, document)?;
    Ok(Report::Promoted {
        path: path.display().to_string(),
        id: args.id.clone(),
    })
}

/// `qctl archive`: move a queued row to the front of the archive, stamped
/// now in the ledger's zone, and take its id out of every `blocked_by`. The
/// row then at the front of the queue becomes active.
///
/// # Errors
///
/// Fails when the ledger does not [`load`], when the id is not queued, when
/// the row is the last one naming an `openspec` change that is still open and
/// `--keep-change-open` gives no reason, when the ledger's `timezone` is not
/// an offset from UTC, or when the file cannot be written.
pub fn archive(args: &ArchiveArgs) -> Result<Report> {
    let path = archive_row(args, None)?;
    Ok(Report::Archived {
        path,
        id: args.id.clone(),
    })
}

/// Archive one queued row, pointing its plan at `plan` when one is given.
fn archive_row(args: &ArchiveArgs, plan: Option<&str>) -> Result<String> {
    let path = resolve_path(&args.ledger);
    let ledger = load(&path)?;
    ensure!(
        ledger.queue.iter().any(|task| task.id == args.id),
        "{} is not queued",
        args.id
    );
    let keep_change_open = args.keep_change_open.as_deref();
    if let Some(reason) = keep_change_open {
        ensure!(
            !reason.trim().is_empty(),
            "--keep-change-open needs a reason"
        );
    } else if let Some(layout) = Layout::read(&path)?
        && let Some(change) = openspec::left_open(&ledger, &layout, &args.id)
    {
        bail!(
            "{id} is the last row naming open change {change}; archive the change first (openspec archive {change}, then qctl close-from-openspec), or pass --keep-change-open REASON",
            id = args.id
        );
    }
    let now = now_in(&ledger.style.timezone)?;

    let mut changes = vec![
        Change::unset("blocked_by"),
        Change::unset("acceptance"),
        Change::set("completed", now.as_str()),
        Change::items("evidence", &args.evidence),
        Change::set("disposition", args.disposition.as_str()),
    ];
    if let Some(plan) = plan {
        changes.push(Change::set("plan", plan));
    }
    if let Some(reason) = keep_change_open {
        changes.push(Change::set("keep_change_open", reason));
    }
    let mut document = read(&path, &ledger.style)?;
    // The row loses what only a queued row carries and gains what only an
    // archived one does on the way across, while it stands alone.
    document.move_between("queue", "archive", &args.id, &changes)?;
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
    let next = document.ids("queue").first().cloned();
    document.set("active", &next)?;
    write(&path, document)?;
    Ok(path.display().to_string())
}

/// `qctl close-from-git`: archive every queued id a commit-body trailer
/// closed, with the commit as its evidence. An id that is the last row of an
/// `openspec` change still open stays queued and is reported as refused.
///
/// # Errors
///
/// Fails when git cannot scan the history, when the pre-push input cannot be
/// read or holds a malformed line, when the `openspec/` folder cannot be
/// listed, when an id fails to [`archive`], or when `active` still names an
/// archived id afterwards.
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
    let layout = Layout::read(&path)?;
    let queued: HashSet<String> = ledger.queue.iter().map(|task| task.id.clone()).collect();
    let mut seen = HashSet::new();
    let mut archived = Vec::new();
    let mut refused = Vec::new();
    for (id, sha) in closed {
        if !queued.contains(&id) || !seen.insert(id.clone()) {
            continue;
        }
        if layout
            .as_ref()
            .and_then(|layout| openspec::left_open(&ledger, layout, &id))
            .is_some()
        {
            refused.push(id);
            continue;
        }
        archive_row(
            &ArchiveArgs {
                id: id.clone(),
                ledger: args.ledger.clone(),
                evidence: vec![sha],
                disposition: Disposition::Completed,
                keep_change_open: None,
            },
            None,
        )?;
        archived.push(id);
    }
    let ledger = load(&path)?;
    if let Some(active) = &ledger.active
        && archived.contains(active)
    {
        bail!("active {active} still names a closed id");
    }
    Ok(Report::ClosedFromGit {
        path: path.display().to_string(),
        retry_push: args.pre_push && !archived.is_empty(),
        archived,
        refused,
    })
}

/// `qctl close-from-openspec`: archive each queued row whose `openspec` change
/// was archived, with the archive folder as its evidence, and point every row
/// that named the change at the same file inside that folder.
///
/// A horizon row and a row whose change was archived more than once are left
/// unchanged and listed for the operator.
///
/// # Errors
///
/// Fails when the ledger does not [`load`], when the `openspec/` folder cannot
/// be listed, when `active` names a row this would archive, or when a row
/// fails to archive or the file cannot be written.
pub fn close_from_openspec(args: &LedgerArgs) -> Result<Report> {
    let path = resolve_path(args);
    let ledger = load(&path)?;
    let root = path.parent().unwrap_or(Path::new(".")).to_path_buf();
    let mut archived = Vec::new();
    let mut repointed = Vec::new();
    let mut left = Vec::new();
    let Some(layout) = Layout::read(&path)? else {
        return Ok(Report::ClosedFromOpenspec {
            path: path.display().to_string(),
            archived,
            repointed,
            left,
        });
    };
    let moved = |plan: Option<&str>| plan.and_then(|plan| openspec::moved(&layout, &root, plan));

    let mut to_archive = Vec::new();
    for task in &ledger.queue {
        match moved(task.plan.as_deref()) {
            Some(Moved::To { folder, plan }) => {
                if ledger.active.as_deref() == Some(task.id.as_str()) {
                    let change = task
                        .plan
                        .as_deref()
                        .and_then(openspec::named)
                        .map_or("", openspec::Named::name);
                    bail!(
                        "active {} names archived change {change}; set active to another row first",
                        task.id
                    );
                }
                to_archive.push((task.id.clone(), folder, plan));
            }
            Some(Moved::Ambiguous(_)) => left.push(task.id.clone()),
            None => {}
        }
    }
    for task in &ledger.horizon {
        if moved(task.plan.as_deref()).is_some() {
            left.push(task.id.clone());
        }
    }
    let mut to_repoint = Vec::new();
    for task in &ledger.archive {
        match moved(task.plan.as_deref()) {
            Some(Moved::To { plan, .. }) => to_repoint.push((task.id.clone(), plan)),
            Some(Moved::Ambiguous(_)) => left.push(task.id.clone()),
            None => {}
        }
    }

    for (id, folder, plan) in to_archive {
        archive_row(
            &ArchiveArgs {
                id: id.clone(),
                ledger: args.clone(),
                evidence: vec![format!("{folder}/")],
                disposition: Disposition::Completed,
                keep_change_open: None,
            },
            Some(&plan),
        )?;
        archived.push(id);
    }
    if !to_repoint.is_empty() {
        let ledger = load(&path)?;
        let mut document = read(&path, &ledger.style)?;
        for (id, plan) in to_repoint {
            let index = ledger
                .archive
                .iter()
                .position(|task| task.id == id)
                .with_context(|| format!("{id} left the archive"))?;
            document.revise("archive", index, &[Change::set("plan", plan.as_str())])?;
            repointed.push(id);
        }
        write(&path, document)?;
    }
    Ok(Report::ClosedFromOpenspec {
        path: path.display().to_string(),
        archived,
        repointed,
        left,
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

/// The ledger file as a document, written in the style it declares.
pub(crate) fn read(path: &Path, style: &Style) -> Result<Document> {
    let source = fs::read_to_string(path).with_context(|| path.display().to_string())?;
    Document::new(source, style)
}

/// Refuse a note holding a line break when the ledger writes each list item
/// on one line, before anything is written: only the author knows where a
/// pasted paragraph's real breaks are.
pub(crate) fn one_line_notes(style: &Style, notes: &[String]) -> Result<()> {
    match style.list_items {
        ListItems::Line => ensure!(
            !notes.iter().any(|note| note.contains(['\n', '\r'])),
            "a note is one line: pass each line as its own -n/--note"
        ),
        ListItems::Auto => {}
    }
    Ok(())
}

/// Write only a document that still reads as the ledger it was, so a verb can
/// never leave a file behind that the next one cannot open.
///
/// A ledger whose style asks for it is normalized on the way out. Off by
/// default, because normalizing touches lines the verb had no business in, and
/// a diff that shows only the work is worth more than one that is always tidy.
pub(crate) fn write(path: &Path, document: Document) -> Result<()> {
    let source = document.into_source();
    crate::document::must_still_parse(&source)?;
    let edited: crate::ledger::Ledger = Input::new(path.display().to_string(), source.as_str())
        .parse()
        .context("read back what this verb wrote")?;
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
