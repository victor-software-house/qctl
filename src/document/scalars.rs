//! `fmt`'s rewrite of a ledger's strings into the layout and quotes its
//! `style` declares, record kind by record kind.

use anyhow::{Context, Result};
use serde::Serialize;
use yamled::{Path, Style, TextStyle};

use super::{Document, notes, section};
use crate::schema::{ArchivedTask, HorizonTask, Ledger, ListItems, QueuedTask};

impl Document {
    /// Write `prefix`, `active`, and every string a row holds on one line in
    /// the declared quotes, and under `line` every block item of a list on one
    /// line, a note one item per line, all with one parse. Each value is read
    /// from `ledger`, not from the source, so no text can drift. Keys, block
    /// row scalars, values that are not strings, and the `style` block stay as
    /// written.
    ///
    /// # Errors
    ///
    /// Fails when yamled refuses an edit or the result does not parse.
    pub fn rewrite_scalars(&mut self, ledger: &Ledger) -> Result<()> {
        let layout = self.layout;
        self.tight(|yaml| {
            yaml.batch(|yaml| -> Result<()> {
                let mut strings = Strings { yaml, layout };
                strings.scalar(&section("prefix"), &ledger.prefix)?;
                if let Some(active) = &ledger.active {
                    strings.scalar(&section("active"), active)?;
                }
                for (index, task) in ledger.queue.iter().enumerate() {
                    strings.queued(&section("queue").index(index), task)?;
                }
                for (index, task) in ledger.archive.iter().enumerate() {
                    strings.archived(&section("archive").index(index), task)?;
                }
                for (index, task) in ledger.horizon.iter().enumerate() {
                    strings.horizon(&section("horizon").index(index), task)?;
                }
                Ok(())
            })
        })
        .context("rewrite the ledger's strings")
    }
}

/// The string edits of one batch. Every path names the document as the batch
/// found it.
struct Strings<'a> {
    yaml: &'a mut yamled::Document,
    layout: ListItems,
}

impl Strings<'_> {
    fn queued(&mut self, row: &Path, task: &QueuedTask) -> Result<()> {
        let QueuedTask {
            id,
            title,
            scope,
            outcome,
            blocked_by,
            acceptance,
            patch,
            plan,
            links,
            notes,
        } = task;
        self.row(
            row,
            &[
                ("id", Some(id)),
                ("title", Some(title)),
                ("scope", Some(scope)),
                ("outcome", Some(outcome)),
                ("patch", patch.as_ref()),
                ("plan", plan.as_ref()),
            ],
            &[
                ("blocked_by", blocked_by),
                ("acceptance", acceptance),
                ("links", links),
                ("notes", notes),
            ],
        )
    }

    fn archived(&mut self, row: &Path, task: &ArchivedTask) -> Result<()> {
        let ArchivedTask {
            id,
            title,
            scope,
            completed,
            outcome,
            evidence,
            disposition,
            patch,
            plan,
            links,
            notes,
        } = task;
        let disposition = variant(disposition)?;
        self.row(
            row,
            &[
                ("id", Some(id)),
                ("title", Some(title)),
                ("scope", Some(scope)),
                ("completed", Some(completed)),
                ("outcome", Some(outcome)),
                ("disposition", Some(&disposition)),
                ("patch", patch.as_ref()),
                ("plan", plan.as_ref()),
            ],
            &[("evidence", evidence), ("links", links), ("notes", notes)],
        )
    }

    fn horizon(&mut self, row: &Path, task: &HorizonTask) -> Result<()> {
        let HorizonTask {
            id,
            title,
            scope,
            outcome,
            kind,
            open,
            patch,
            plan,
            links,
            notes,
        } = task;
        let kind = variant(kind)?;
        self.row(
            row,
            &[
                ("id", Some(id)),
                ("title", Some(title)),
                ("scope", Some(scope)),
                ("outcome", Some(outcome)),
                ("kind", Some(&kind)),
                ("open", Some(open)),
                ("patch", patch.as_ref()),
                ("plan", plan.as_ref()),
            ],
            &[("links", links), ("notes", notes)],
        )
    }

    fn row(
        &mut self,
        row: &Path,
        scalars: &[(&str, Option<&String>)],
        lists: &[(&str, &Vec<String>)],
    ) -> Result<()> {
        for (key, text) in scalars {
            if let Some(text) = text {
                self.scalar(&row.clone().key(*key), text)?;
            }
        }
        for (key, items) in lists {
            self.list(&row.clone().key(*key), key, items)?;
        }
        Ok(())
    }

    /// Under `line`, a block item becomes one line, and a note whose text
    /// holds line breaks becomes one item per line, each further line
    /// inserted after the first in order. Every other item is re-quoted.
    fn list(&mut self, list: &Path, key: &str, items: &[String]) -> Result<()> {
        let Some(node) = self.yaml.node(list) else {
            return Ok(());
        };
        let split = node.style() == Style::BlockSequence && self.layout == ListItems::Line;
        for (at, item) in items.iter().enumerate() {
            let path = list.clone().index(at);
            let lines = if key == "notes" {
                notes::lines(item)
            } else {
                vec![item.as_str()]
            };
            let block = self.styled(&path, &[Style::Literal, Style::Folded]);
            if !split || (!block && lines == [item.as_str()]) {
                self.scalar(&path, item)?;
                continue;
            }
            let mut lines = lines.into_iter();
            if let Some(first) = lines.next() {
                self.yaml.replace_text(&path, first, TextStyle::Line)?;
            }
            for line in lines {
                self.yaml
                    .insert_item_text(list, at + 1, line, TextStyle::Line)?;
            }
        }
        Ok(())
    }

    /// A string written on one line, written again in the declared quotes. A
    /// block scalar keeps its layout; yamled skips a write of the same bytes.
    fn scalar(&mut self, path: &Path, text: &str) -> Result<()> {
        if self.styled(
            path,
            &[Style::Plain, Style::SingleQuoted, Style::DoubleQuoted],
        ) {
            self.yaml.replace_text(path, text, TextStyle::Line)?;
        }
        Ok(())
    }

    fn styled(&self, path: &Path, styles: &[Style]) -> bool {
        self.yaml
            .node(path)
            .is_some_and(|node| styles.contains(&node.style()))
    }
}

/// The name serde writes for a unit variant, such as `completed`.
fn variant(value: &impl Serialize) -> Result<String> {
    serde_json::to_value(value)?
        .as_str()
        .map(str::to_owned)
        .context("a unit variant serializes as a string")
}
