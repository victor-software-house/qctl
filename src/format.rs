//! A ledger rewritten into the style it declares.
//!
//! Every rule here is named by an option in [`Style`] or by a rule `check`
//! already enforces. Nothing is inferred from how the file happens to look: a
//! ledger says how it wants to be written, and this makes it so.
//!
//! What it does not do is add anything nobody asked for. It removes whitespace
//! no one chose — a space at the end of a line, a second blank line, a file that
//! does not end in exactly one newline — and it never inserts a blank line, a
//! comment, or a key. Row key order and quote style are the obvious next
//! options, and are deliberately not decided here yet.

use crate::cli::FmtArgs;
use crate::document::{Document, KeyShape, must_still_parse};
use crate::ledger::{Ledger, resolve_path};
use crate::report::Report;
use crate::schema::{ArchiveOrder, Section, VERSION};
use anyhow::{Context, Result, bail};
use std::fs;
use yaml_serde::Value;

/// The ledger as its own style says it should be written.
pub fn normalized(source: &str, ledger: &Ledger) -> Result<String> {
    let style = &ledger.style;
    let mut document = Document::new(source.to_owned());

    if style.archive_order == ArchiveOrder::NewestFirst {
        document.reorder_rows("archive", &newest_first(ledger))?;
    }
    for section in Section::ALL {
        document.set_indent(section.key(), style.indent.into())?;
    }
    let order: Vec<&str> = style.section_order.iter().map(Section::key).collect();
    document.reorder_sections(&order)?;

    let source = tidied(&document.into_source());
    must_still_parse(&source)?;
    Ok(source)
}

/// The archive's ids, most recently completed first. Stamps share one zone and
/// one shape, so comparing their text compares their moments.
fn newest_first(ledger: &Ledger) -> Vec<String> {
    let mut rows: Vec<&crate::schema::ArchivedTask> = ledger.archive.iter().collect();
    rows.sort_by(|left, right| right.completed.cmp(&left.completed));
    rows.into_iter().map(|row| row.id.clone()).collect()
}

/// Whitespace nobody chose: a space at the end of a line, a second blank line, a
/// blank line between a list's key and its first row, and a file that does not
/// end in exactly one newline.
fn tidied(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut blank_run = 0;
    let mut after_key = false;
    for line in source.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            blank_run += 1;
            if blank_run > 1 || after_key {
                continue;
            }
            out.push('\n');
            continue;
        }
        blank_run = 0;
        after_key = line.ends_with(':') && !line.starts_with(char::is_whitespace);
        out.push_str(line);
        out.push('\n');
    }
    while out.ends_with("\n\n") {
        out.pop();
    }
    out
}

/// `qctl fmt`: write the ledger in its declared style, or with `--check` say
/// what is not in it and leave the file alone. A schema 3 file is rewritten
/// to schema 4 first: each scalar `notes` becomes a list of paragraphs.
pub fn run(args: &FmtArgs) -> Result<Report> {
    let path = resolve_path(&args.ledger);
    let original = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let source = upgrade_v3(&original)?;
    let ledger: Ledger =
        serde_yml::from_str(&source).with_context(|| format!("parse {}", path.display()))?;
    let complaints = crate::ledger::value_errors(&ledger);
    anyhow::ensure!(
        complaints.is_empty(),
        "validate {}: {}",
        path.display(),
        complaints.join("; ")
    );
    let wanted = normalized(&source, &ledger)?;

    if original == wanted {
        return Ok(Report::Formatted {
            path: path.display().to_string(),
            changed: false,
            check: args.check,
            differences: Vec::new(),
        });
    }
    let differences = changes(&original, &wanted);
    if !args.check {
        fs::write(&path, wanted).with_context(|| format!("write {}", path.display()))?;
    }
    Ok(Report::Formatted {
        path: path.display().to_string(),
        changed: true,
        check: args.check,
        differences,
    })
}

/// Which lines `fmt` would change, by number, so `--check` says where to look
/// rather than only that something is wrong.
fn changes(source: &str, wanted: &str) -> Vec<String> {
    let (from, to): (Vec<&str>, Vec<&str>) = (source.lines().collect(), wanted.lines().collect());
    if from.len() != to.len() {
        return vec![format!("{} lines, would be {}", from.len(), to.len())];
    }
    from.iter()
        .zip(&to)
        .enumerate()
        .filter(|(_, (before, after))| before != after)
        .map(|(at, (before, after))| format!("line {}: {:?} would be {:?}", at + 1, before, after))
        .collect()
}

/// A schema 3 ledger becomes schema 4: scalar `notes` split on blank-line
/// paragraphs, then `schema_version` is set to 4. A current-version file is
/// returned unchanged. Other versions are refused.
fn upgrade_v3(source: &str) -> Result<String> {
    match peek_schema_version(source) {
        Some(version) if version == u64::from(VERSION) => Ok(source.to_owned()),
        Some(3) => rewrite_v3_notes(source),
        Some(version) => {
            bail!("schema_version {version} must be {VERSION} (or 3, which fmt rewrites)")
        }
        None => Ok(source.to_owned()),
    }
}

fn peek_schema_version(source: &str) -> Option<u64> {
    let value: serde_yml::Value = serde_yml::from_str(source).ok()?;
    value.get("schema_version")?.as_u64()
}

fn rewrite_v3_notes(source: &str) -> Result<String> {
    let parsed: serde_yml::Value =
        serde_yml::from_str(source).context("parse a schema 3 ledger")?;
    let mut document = Document::new(source.to_owned());
    for section in ["queue", "archive", "horizon"] {
        let Some(rows) = parsed.get(section).and_then(serde_yml::Value::as_sequence) else {
            continue;
        };
        for (index, row) in rows.iter().enumerate() {
            let Some(notes) = row.get("notes") else {
                continue;
            };
            if document.row_key_shape(section, index, "notes")? != Some(KeyShape::Scalar) {
                continue;
            }
            let Some(text) = notes.as_str() else {
                continue;
            };
            let items = paragraphs(text);
            if items.is_empty() {
                document.remove_row_key(section, index, "notes")?;
            } else {
                document.replace_row_value(
                    section,
                    index,
                    "notes",
                    &yaml_serde::to_value(&items)?,
                )?;
            }
        }
    }
    document.set("schema_version", Value::from(VERSION))?;
    Ok(document.into_source())
}

fn paragraphs(notes: &str) -> Vec<String> {
    notes
        .split("\n\n")
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}
