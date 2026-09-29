//! A ledger as the text somebody wrote, edited where it must change and copied
//! everywhere else.
//!
//! [`yamled`] does the editing: it finds each node by path, moves a row with
//! the comments it owns, closes a list that loses its last row, and keeps a
//! rewritten list in the style it was written in. What stays here is the
//! ledger's vocabulary: sections, rows found by id, and how a note is written.

mod notes;
mod rows;

use anyhow::{Context, Result, bail};
use serde::Serialize;
use yamled::{Path, Segment, Spacing, Style};

/// A ledger's text, parsed so it can be edited in place.
pub struct Document {
    yaml: yamled::Document,
}

/// One change to a row's fields.
#[derive(Clone, Debug)]
pub enum Change {
    /// A value written in place, or added at the end of the row.
    Set {
        key: String,
        value: serde_json::Value,
    },
    /// A key taken off the row.
    Unset { key: String },
    /// A list of strings rewritten. Items appended to a block list are added
    /// after the ones already written, so those keep their bytes.
    List {
        key: String,
        before: Vec<String>,
        after: Vec<String>,
    },
}

impl Change {
    /// Write `value` under `key`.
    pub fn set(key: &str, value: impl Into<serde_json::Value>) -> Self {
        Self::Set {
            key: key.to_owned(),
            value: value.into(),
        }
    }

    /// Take `key` off the row.
    #[must_use]
    pub fn unset(key: &str) -> Self {
        Self::Unset {
            key: key.to_owned(),
        }
    }
}

impl Document {
    /// A ledger's text. A row added to a list whose own rows do not settle
    /// its spacing is set off by a blank line, as every ledger qctl writes.
    pub fn new(source: String) -> Result<Self> {
        let yaml = yamled::Document::parse(source)
            .context("this ledger is not YAML qctl can edit")?
            .with_spacing(Spacing::Blank);
        Ok(Self { yaml })
    }

    #[must_use]
    pub fn into_source(self) -> String {
        self.yaml.into_string()
    }

    /// Put the lists in the order these names give, each with the comments it
    /// owns. A list the file does not have is skipped; everything that is not
    /// a list stays in its slot.
    pub fn reorder_sections(&mut self, order: &[&str]) -> Result<()> {
        let present: Vec<Segment> = order
            .iter()
            .filter(|name| self.yaml.node(&section(name)).is_some())
            .map(|name| Segment::Key((*name).to_owned()))
            .collect();
        if present.len() < 2 {
            return Ok(());
        }
        self.yaml
            .reorder(&Path::root(), &present)
            .context("reorder the lists")
    }

    /// Move every row of a section to sit this far under its key. A section
    /// with no rows has nothing to move.
    pub fn set_indent(&mut self, name: &str, indent: usize) -> Result<()> {
        let path = section(name);
        let Some(list) = self.yaml.node(&path) else {
            return Ok(());
        };
        if list.style() != Style::BlockSequence || list.is_empty() {
            return Ok(());
        }
        let Some(first) = self.yaml.locate(&path.clone().index(0)) else {
            return Ok(());
        };
        let source = self.yaml.as_str();
        let line = &source[source[..first.start].rfind('\n').map_or(0, |at| at + 1)..];
        if line.len() - line.trim_start().len() == indent {
            return Ok(());
        }
        self.yaml
            .reindent(&path, indent)
            .with_context(|| format!("re-indent {name}"))
    }

    /// Replace a top-level value, leaving its line where it was.
    pub fn set<T: Serialize + ?Sized>(&mut self, key: &str, value: &T) -> Result<()> {
        self.yaml
            .replace(&section(key), value)
            .with_context(|| format!("write {key}"))
    }

    /// Whether this row has `key`, and whether its value is a sequence.
    #[must_use]
    pub fn row_key_shape(&self, name: &str, index: usize, key: &str) -> Option<KeyShape> {
        let node = self.yaml.node(&section(name).index(index).key(key))?;
        Some(match node.style() {
            Style::BlockSequence | Style::FlowSequence => KeyShape::Sequence,
            _ => KeyShape::Scalar,
        })
    }

    /// Drop a key from a row that is still in the file.
    pub fn remove_row_key(&mut self, name: &str, index: usize, key: &str) -> Result<()> {
        self.yaml
            .remove(&section(name).index(index).key(key))
            .with_context(|| format!("remove {key} from {name}[{index}]"))
    }

    /// Write a row's notes as a list, each note in the style [`notes::style`]
    /// picks. Used by `fmt` when rewriting a v3 scalar `notes`.
    pub fn replace_row_notes(&mut self, name: &str, index: usize, items: &[String]) -> Result<()> {
        let path = section(name).index(index).key("notes");
        self.yaml
            .replace(&path, &NO_ITEMS)
            .context("empty the notes")?;
        self.push_notes(&path, items)
    }

    /// Apply field changes to one row.
    pub fn revise(&mut self, name: &str, index: usize, changes: &[Change]) -> Result<()> {
        let row = section(name).index(index);
        for change in changes {
            match change {
                Change::Set { key, value } => self.write_field(&row, key, value)?,
                Change::Unset { key } => self.drop_field(&row, key)?,
                Change::List { key, before, after } => {
                    self.write_list(&row, key, before, after)?;
                }
            }
        }
        Ok(())
    }

    fn write_field(&mut self, row: &Path, key: &str, value: &serde_json::Value) -> Result<()> {
        let path = row.clone().key(key);
        if self.yaml.node(&path).is_some() {
            self.yaml.replace(&path, value)
        } else {
            self.yaml.insert(row, key, value, yamled::Position::End)
        }
        .with_context(|| format!("write {key}"))
    }

    fn drop_field(&mut self, row: &Path, key: &str) -> Result<()> {
        let path = row.clone().key(key);
        if self.yaml.node(&path).is_none() {
            return Ok(());
        }
        self.yaml
            .remove(&path)
            .with_context(|| format!("remove {key}"))
    }

    fn write_list(
        &mut self,
        row: &Path,
        key: &str,
        before: &[String],
        after: &[String],
    ) -> Result<()> {
        if after.is_empty() {
            return self.drop_field(row, key);
        }
        let path = row.clone().key(key);
        let Some(list) = self.yaml.node(&path) else {
            self.yaml
                .insert(row, key, &NO_ITEMS, yamled::Position::End)
                .with_context(|| format!("add {key}"))?;
            return self.push_items(&path, key, after);
        };
        let appended = list.style() == Style::BlockSequence && after.starts_with(before);
        if appended {
            return self.push_items(&path, key, &after[before.len()..]);
        }
        if key == "notes" {
            self.yaml
                .replace(&path, &NO_ITEMS)
                .context("empty the notes")?;
            return self.push_notes(&path, after);
        }
        self.yaml
            .replace(&path, after)
            .with_context(|| format!("write {key}"))
    }

    fn push_items(&mut self, path: &Path, key: &str, items: &[String]) -> Result<()> {
        if key == "notes" {
            return self.push_notes(path, items);
        }
        self.tight(|yaml| {
            items
                .iter()
                .try_for_each(|item| yaml.push(path, item))
                .with_context(|| format!("add to {key}"))
        })
    }

    fn push_notes(&mut self, path: &Path, items: &[String]) -> Result<()> {
        self.tight(|yaml| {
            items
                .iter()
                .try_for_each(|note| yaml.push_text(path, note, notes::style(note)))
                .context("add a note")
        })
    }

    /// Run an edit with items added tight: only rows are set off by a blank
    /// line, and a row's own lists never are.
    fn tight<R>(&mut self, edit: impl FnOnce(&mut yamled::Document) -> R) -> R {
        let mut yaml = self.yaml.clone().with_spacing(Spacing::Tight);
        let result = edit(&mut yaml);
        self.yaml = yaml.with_spacing(Spacing::Blank);
        result
    }
}

/// An empty list, for a key that is about to be filled.
const NO_ITEMS: [&str; 0] = [];

/// How a row field is written.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyShape {
    /// A string, number, or folded/literal scalar.
    Scalar,
    /// A block or flow sequence.
    Sequence,
}

/// The path of a top-level key.
fn section(name: &str) -> Path {
    Path::root().key(name)
}

/// Fail rather than write something the next verb cannot read.
pub fn must_still_parse(source: &str) -> Result<()> {
    if ctl_core::input::Input::new("tasks.yaml", source)
        .parse::<serde_json::Value>()
        .is_err()
    {
        bail!("this edit would leave a file that is not valid YAML");
    }
    Ok(())
}
