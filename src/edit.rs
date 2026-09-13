//! Surgical edits to one existing queue, horizon, or archive row.

mod fields;
mod lists;
mod policy;
mod position;

use crate::cli::{EditArgs, RemoveField};
use crate::document::revise_fields;
use crate::ledger::{Ledger, load, resolve_path};
use crate::report::Report;
use anyhow::{Result, bail, ensure};
use fields::ListEdit;
use policy::RowKind;
use std::path::Path;

pub fn run(args: &EditArgs) -> Result<Report> {
    let requested = policy::requested(args);
    ensure!(!requested.is_empty(), "edit needs a field or a position");
    let path = resolve_path(&args.ledger);
    let ledger = load(&path)?;
    let located = locate(&ledger, &args.id)?;
    policy::validate(&requested, located.kind)?;
    let row = RowLists::new(&ledger, located);
    let removals = lists::Removals::new(args);

    let notes = lists::apply(
        row.notes,
        &args.note,
        removals.for_field(RemoveField::Note),
        args.reorder_notes.as_deref(),
    )?;
    let links = lists::apply(
        row.links,
        &args.links,
        removals.for_field(RemoveField::Link),
        None,
    )?;
    let mut changed_lists = vec![
        ListEdit::new("notes", row.notes, notes),
        ListEdit::new("links", row.links, links),
    ];

    let mut edited_blockers = None;
    if let Some(current) = row.acceptance {
        let acceptance = lists::apply(
            current,
            &args.acceptance,
            removals.for_field(RemoveField::Acceptance),
            None,
        )?;
        ensure!(!acceptance.is_empty(), "queued rows need acceptance");
        changed_lists.push(ListEdit::new("acceptance", current, acceptance));
    }
    if let Some(current) = row.blocked_by {
        let blocked_by = lists::apply(
            current,
            &args.blocked_by,
            removals.for_field(RemoveField::BlockedBy),
            None,
        )?;
        changed_lists.push(ListEdit::new("blocked_by", current, blocked_by.clone()));
        edited_blockers = Some(blocked_by);
    }
    if let Some(current) = row.evidence {
        let evidence = lists::apply(
            current,
            &args.evidence,
            removals.for_field(RemoveField::Evidence),
            None,
        )?;
        ensure!(!evidence.is_empty(), "archived rows need evidence");
        changed_lists.push(ListEdit::new("evidence", current, evidence));
    }

    let changes = fields::changes(args, &path, &changed_lists)?;
    let destination = if located.kind == RowKind::Queue {
        position::destination(&ledger, args, edited_blockers.as_deref())?
    } else {
        located.index
    };
    rewrite_row(
        &path,
        located.kind.section(),
        &args.id,
        destination,
        &changes,
    )?;
    Ok(Report::Edited {
        path: path.display().to_string(),
        id: args.id.clone(),
    })
}

#[derive(Clone, Copy)]
struct Located {
    kind: RowKind,
    index: usize,
}

fn locate(ledger: &Ledger, id: &str) -> Result<Located> {
    if let Some(index) = ledger.queue.iter().position(|task| task.id == id) {
        return Ok(Located {
            kind: RowKind::Queue,
            index,
        });
    }
    if let Some(index) = ledger.horizon.iter().position(|task| task.id == id) {
        return Ok(Located {
            kind: RowKind::Horizon,
            index,
        });
    }
    if let Some(index) = ledger.archive.iter().position(|task| task.id == id) {
        return Ok(Located {
            kind: RowKind::Archive,
            index,
        });
    }
    bail!("{id} is not in the ledger")
}

struct RowLists<'a> {
    notes: &'a [String],
    links: &'a [String],
    acceptance: Option<&'a [String]>,
    blocked_by: Option<&'a [String]>,
    evidence: Option<&'a [String]>,
}

impl<'a> RowLists<'a> {
    fn new(ledger: &'a Ledger, located: Located) -> Self {
        match located.kind {
            RowKind::Queue => {
                let row = &ledger.queue[located.index];
                Self {
                    notes: &row.notes,
                    links: &row.links,
                    acceptance: Some(&row.acceptance),
                    blocked_by: Some(&row.blocked_by),
                    evidence: None,
                }
            }
            RowKind::Horizon => {
                let row = &ledger.horizon[located.index];
                Self {
                    notes: &row.notes,
                    links: &row.links,
                    acceptance: None,
                    blocked_by: None,
                    evidence: None,
                }
            }
            RowKind::Archive => {
                let row = &ledger.archive[located.index];
                Self {
                    notes: &row.notes,
                    links: &row.links,
                    acceptance: None,
                    blocked_by: None,
                    evidence: Some(&row.evidence),
                }
            }
        }
    }
}

fn rewrite_row(
    path: &Path,
    section: &str,
    id: &str,
    destination: usize,
    changes: &[(String, Option<yaml_serde::Value>)],
) -> Result<()> {
    let mut document = crate::mutate::read(path)?;
    let current = document.position_of(section, id)?;
    if changes.is_empty() && current == destination {
        return Ok(());
    }
    let row = document.cut(section, id)?;
    let row = if changes.is_empty() {
        row
    } else {
        let refs: Vec<(&str, Option<yaml_serde::Value>)> = changes
            .iter()
            .map(|(key, value)| (key.as_str(), value.clone()))
            .collect();
        revise_fields(&row, &refs)?
    };
    document.paste_at(section, destination, &row)?;
    crate::mutate::write(path, document)
}
