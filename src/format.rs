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
use crate::document::{Document, must_still_parse};
use crate::ledger::{Ledger, load, resolve_path};
use crate::schema::{ArchiveOrder, Section};
use anyhow::{Context, Result, bail};
use std::fs;

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
/// what is not in it and leave the file alone.
pub fn run(args: &FmtArgs) -> Result<()> {
    let path = resolve_path(&args.ledger);
    let ledger = load(&path)?;
    let source = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let wanted = normalized(&source, &ledger)?;

    if source == wanted {
        println!("ok  {}", path.display());
        return Ok(());
    }
    if args.check {
        for line in changes(&source, &wanted) {
            eprintln!("qctl: {line}");
        }
        bail!("{} is not in its declared style", path.display());
    }
    fs::write(&path, wanted).with_context(|| format!("write {}", path.display()))?;
    println!("wrote {}", path.display());
    Ok(())
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
