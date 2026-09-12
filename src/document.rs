//! A ledger as the text somebody wrote, edited where it must change and copied
//! everywhere else.
//!
//! The structure comes from a real YAML parser: [`yamlpath`] gives the byte span
//! of any row and [`yamlpatch`] performs the key-level edits, so nothing here
//! guesses at indentation or decides how to quote a value.
//!
//! Two things are ours, because the format leaves them open. A row moves by
//! byte range, since no patch operation offers a move. And a comment written
//! directly above a row — no blank line between — belongs to that row: the
//! parser hands a comment to the row *above* it, which would strand a comment
//! on one move and steal it on the next.
//!
//! That same attribution is why a row is edited while it is out of the file. A
//! removal span for the last key of a row reaches past it, so removing
//! `acceptance` from a row would take the following row's comment with it. A row
//! on its own has no neighbour to rob.

mod rows;
mod text;
mod value;

use anyhow::{Context, Result, bail};
use std::ops::Range;
use text::{dedent, indent_by};
use yaml_serde::Value;
use yamlpatch::{Op, Patch, apply_yaml_patches};
use yamlpath::{Document as Parsed, Route, route};

/// How deep a section's rows sit. Every ledger this tool has written uses two
/// spaces; a file that says otherwise is followed rather than corrected.
const DEFAULT_INDENT: usize = 2;

/// A ledger's text, with the parser's view of it available on demand.
pub struct Document {
    source: String,
}

impl Document {
    #[must_use]
    pub fn new(source: String) -> Self {
        Self { source }
    }

    #[must_use]
    pub fn into_source(self) -> String {
        self.source
    }

    fn parsed(&self) -> Result<Parsed> {
        Parsed::new(self.source.clone()).context("this ledger is not YAML the parser can follow")
    }

    /// Every id a section's rows carry, in file order.
    /// Put the lists in the order these names give, moving each one whole: its
    /// key line, its rows, and the comments and blank lines between them.
    ///
    /// Everything above the first list — the version, the prefix, the style, the
    /// active row — stays where it is.
    pub fn reorder_sections(&mut self, order: &[&str]) -> Result<()> {
        // Only the lists this file has: `horizon` is optional, and a ledger
        // without one is still a ledger.
        let mut blocks = Vec::with_capacity(order.len());
        for section in order {
            if let Some(span) = self.block_of(section)? {
                blocks.push((*section, span));
            }
        }
        if blocks.len() < 2 {
            return Ok(());
        }
        let mut found: Vec<Range<usize>> = blocks.iter().map(|(_, span)| span.clone()).collect();
        found.sort_by_key(|span| span.start);
        let wanted: Vec<&str> = blocks.iter().map(|(section, _)| *section).collect();
        let already: Vec<&str> = {
            let mut named: Vec<(usize, &str)> = blocks
                .iter()
                .map(|(section, span)| (span.start, *section))
                .collect();
            named.sort_by_key(|(start, _)| *start);
            named.into_iter().map(|(_, section)| section).collect()
        };
        // Nothing below moves anything, so a file already in its order is done —
        // and must not be refused for a comment that only a move would disturb.
        if already == wanted {
            return Ok(());
        }
        for pair in found.windows(2) {
            if pair[0].end > pair[1].start {
                bail!("these lists overlap, which is not a file this can reorder");
            }
            // A comment directly above a key belongs to that list and moves with
            // it. Anything else out here belongs to no list, and moving the
            // lists around it would either take it along or lose it. Refuse
            // instead: better a ledger this will not reorder than a comment
            // silently gone.
            if !self.source[pair[0].end..pair[1].start].trim().is_empty() {
                bail!(
                    "something between these lists belongs to neither; move it above a key or out of the way"
                );
            }
        }
        // The blank lines between the lists are the file's own spacing, so they
        // stay in the sequence they were written in. Only the lists move.
        let gaps: Vec<String> = found
            .windows(2)
            .map(|pair| self.source[pair[0].end..pair[1].start].to_owned())
            .collect();
        let mut rebuilt = String::new();
        for (at, (_, span)) in blocks.iter().enumerate() {
            rebuilt.push_str(self.source[span.clone()].trim_end());
            if let Some(gap) = gaps.get(at) {
                rebuilt.push_str(gap);
            }
        }
        let (from, to) = (found[0].start, found[found.len() - 1].end);
        self.source.replace_range(from..to, &rebuilt);
        Ok(())
    }

    /// Move every row of a section to sit this far under its key.
    pub fn set_indent(&mut self, section: &str, indent: usize) -> Result<()> {
        loop {
            let Some(row) = self
                .rows(section)?
                .into_iter()
                .find(|row| self.depth_of(row.start) != indent)
            else {
                return Ok(());
            };
            let depth = self.depth_of(row.start);
            let text = self.source[row.clone()].to_owned();
            // A row's span runs to the next row's first line, so it carries the
            // blank lines that separate them — and the last row's span stops at
            // the end of its text. Whatever ended the span has to end it still.
            let (body, tail) = text.split_at(text.trim_end().len());
            let moved = indent_by(&dedent(body, depth), indent);
            self.source
                .replace_range(row.start..row.end, &format!("{moved}{tail}"));
        }
    }

    /// A section from the comment above its key through its last row, without
    /// the blank lines or comments that follow it. `None` when the file has no
    /// such list, which `horizon` is allowed to be.
    fn block_of(&self, section: &str) -> Result<Option<Range<usize>>> {
        if self.parsed()?.query_key_only(&route!(section)).is_err() {
            return Ok(None);
        }
        let (key_start, key_line_end) = self.key_line(section)?;
        let from = self.line_holding(key_start);
        let end = self
            .rows(section)?
            .last()
            .map_or(key_line_end, |last| last.end);
        Ok(Some(from..end.max(key_line_end)))
    }

    /// How far the line at this offset is indented.
    fn depth_of(&self, offset: usize) -> usize {
        let line = &self.source[offset..];
        line.len() - line.trim_start().len()
    }

    /// Replace a top-level scalar, leaving its line where it was.
    pub fn set(&mut self, key: &str, value: Value) -> Result<()> {
        self.patch(&[Patch {
            route: route!(key),
            operation: Op::Replace(value),
        }])
    }

    /// Apply patch operations, stated against routes rather than offsets, so
    /// they cannot invalidate one another.
    fn patch(&mut self, patches: &[Patch]) -> Result<()> {
        let parsed = self.parsed()?;
        let edited = apply_yaml_patches(&parsed, patches).context("edit this ledger")?;
        edited.source().clone_into(&mut self.source);
        Ok(())
    }

    /// Replace one key on a row that is still in the file. Used by `fmt` when
    /// rewriting a v3 scalar `notes` into a list without moving the row.
    pub fn replace_row_value(
        &mut self,
        section: &str,
        index: usize,
        key: &str,
        value: &Value,
    ) -> Result<()> {
        self.replace_value(&route!(section, index, key), value)
    }

    fn replace_value(&mut self, route: &Route, value: &Value) -> Result<()> {
        if matches!(value, Value::Sequence(_)) {
            let span = {
                let parsed = self.parsed()?;
                let feature = parsed
                    .query_exact(route)?
                    .context("the value to replace is missing")?;
                let (from, to) = feature.location.byte_span;
                from..to
            };
            let written = value::sequence_like(&self.source, &span, value)?;
            self.source.replace_range(span, &written);
            return Ok(());
        }
        self.patch(&[Patch {
            route: route.clone(),
            operation: Op::Replace(value.clone()),
        }])
    }

    /// Drop a key from a row that is still in the file.
    pub fn remove_row_key(&mut self, section: &str, index: usize, key: &str) -> Result<()> {
        self.patch(&[Patch {
            route: route!(section, index, key),
            operation: Op::Remove,
        }])
    }

    /// Whether this row has `key`, and whether its value is a YAML sequence.
    pub fn row_key_shape(
        &self,
        section: &str,
        index: usize,
        key: &str,
    ) -> Result<Option<KeyShape>> {
        let parsed = self.parsed()?;
        let Ok(Some(feature)) = parsed.query_exact(&route!(section, index, key)) else {
            return Ok(None);
        };
        let text = parsed.extract(&feature).trim();
        if text.starts_with('[') || text.starts_with('-') {
            return Ok(Some(KeyShape::Sequence));
        }
        Ok(Some(KeyShape::Scalar))
    }
}

/// How a row field is written.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyShape {
    /// A string, number, or folded/literal scalar.
    Scalar,
    /// A block or flow sequence.
    Sequence,
}

/// Drop keys from one row and add others, with the row standing alone so no
/// removal can reach a neighbour's comment.
fn revise(row: &str, drop: &[&str], add: &[(&str, Value)]) -> Result<String> {
    let mut changes = Vec::with_capacity(drop.len() + add.len());
    changes.extend(drop.iter().map(|key| (*key, None)));
    changes.extend(add.iter().map(|(key, value)| (*key, Some(value.clone()))));
    revise_fields(row, &changes)
}

/// Drop, replace, or add keys on a row standing alone so a neighbour's comment
/// cannot be taken. Existing keys stay where they are; new keys are appended.
pub fn revise_fields(row: &str, changes: &[(&str, Option<Value>)]) -> Result<String> {
    let mut kept = row.to_owned();
    for (key, value) in changes {
        let document = Parsed::new(kept.clone()).context("read the row on its own")?;
        let exists = document.query_exists(&route!(0, *key));
        match (exists, value) {
            (true, None) => {
                let mut document = Document::new(kept);
                document.patch(&[Patch {
                    route: route!(0, *key),
                    operation: Op::Remove,
                }])?;
                kept = document.into_source();
            }
            (true, Some(value)) => {
                let mut document = Document::new(kept);
                document.replace_value(&route!(0, *key), value)?;
                kept = document.into_source();
            }
            (false, Some(value)) => {
                kept = format!(
                    "{}\n{}",
                    kept.trim_end(),
                    indent_by(&value::mapping_entry(key, value)?, 2)
                );
            }
            (false, None) => {}
        }
    }
    Ok(kept.trim_end().to_owned())
}

/// Fail rather than write something the next verb cannot read.
pub fn must_still_parse(source: &str) -> Result<()> {
    if serde_yml::from_str::<serde_yml::Value>(source).is_err() {
        bail!("this edit would leave a file that is not valid YAML");
    }
    Ok(())
}
