//! Rows found by id, moved between and within sections.
//!
//! A row moves with the comments written directly above it. A list that loses
//! its last row becomes `[]`, and a row put into `[]` opens it as a block list.
//! yamled owns all three rules.

use std::ops::Range;

use anyhow::{Context, Result};
use serde::Serialize;
use yamled::{Path, Position, Segment};

use super::{Change, Document, NO_ITEMS, section};

impl Document {
    /// Every id a section's rows carry, in file order.
    #[must_use]
    pub fn ids(&self, name: &str) -> Vec<String> {
        let path = section(name);
        let count = self.yaml.node(&path).map_or(0, |list| list.len());
        (0..count)
            .filter_map(|index| {
                let id = self.yaml.node(&path.clone().index(index).key("id"))?;
                Some(id.text().trim_matches(['"', '\'']).to_owned())
            })
            .collect()
    }

    /// The lines each of a section's rows owns, in file order: the comments
    /// above it, its entry, and its value. Each range starts and ends at a
    /// line start.
    #[must_use]
    pub fn row_lines(&self, name: &str) -> Vec<Range<usize>> {
        let path = section(name);
        let count = self.yaml.node(&path).map_or(0, |list| list.len());
        (0..count)
            .filter_map(|index| {
                let owned = self.yaml.node(&path.clone().index(index))?.owned();
                Some(owned.start..owned.end)
            })
            .collect()
    }

    /// Where the row carrying `id` sits in its section.
    ///
    /// # Errors
    ///
    /// Fails when no row in the section carries `id`.
    pub fn position_of(&self, name: &str, id: &str) -> Result<usize> {
        self.ids(name)
            .iter()
            .position(|carried| carried == id)
            .with_context(|| format!("{id} is not in {name}"))
    }

    /// Move a row to the front of its section.
    ///
    /// # Errors
    ///
    /// Fails as [`Self::move_row`] does.
    pub fn move_to_front(&mut self, name: &str, id: &str) -> Result<()> {
        self.move_row(name, id, 0)
    }

    /// Move a row within its section so it ends up at `destination`.
    ///
    /// # Errors
    ///
    /// Fails when no row in the section carries `id`, or the row cannot be
    /// moved.
    pub fn move_row(&mut self, name: &str, id: &str, destination: usize) -> Result<()> {
        let from = self.position_of(name, id)?;
        if from == destination {
            return Ok(());
        }
        let row = self.take(name, from)?;
        self.put(name, destination, &row)
    }

    /// Move a row to the front of another section, then change its fields.
    ///
    /// # Errors
    ///
    /// Fails when no row in `from` carries `id`, or the row cannot be moved
    /// or changed.
    pub fn move_between(
        &mut self,
        from: &str,
        to: &str,
        id: &str,
        changes: &[Change],
    ) -> Result<()> {
        self.carry(from, to, id, Some(0), changes)
    }

    /// Move a row to the end of another section, then change its fields.
    ///
    /// # Errors
    ///
    /// Fails when no row in `from` carries `id`, or the row cannot be moved
    /// or changed.
    pub fn move_to_end(
        &mut self,
        from: &str,
        to: &str,
        id: &str,
        changes: &[Change],
    ) -> Result<()> {
        self.carry(from, to, id, None, changes)
    }

    fn carry(
        &mut self,
        from: &str,
        to: &str,
        id: &str,
        at: Option<usize>,
        changes: &[Change],
    ) -> Result<()> {
        let row = self.take(from, self.position_of(from, id)?)?;
        self.ensure_section(to)?;
        let index = at.unwrap_or_else(|| self.ids(to).len());
        self.put(to, index, &row)?;
        self.revise(to, index, changes)
    }

    /// Add a row at the end of a section. yamled writes the row whole, then
    /// each list that holds items is written again by the one item writer, so
    /// its items take the declared layout and the key keeps its slot.
    ///
    /// # Errors
    ///
    /// Fails when the row does not serialize or an item cannot be written.
    pub fn append<T: Serialize>(&mut self, name: &str, row: &T) -> Result<()> {
        self.ensure_section(name)?;
        let list = section(name);
        self.yaml.push(&list, row).context("add the row")?;
        let path = list.index(self.ids(name).len() - 1);
        let fields = serde_json::to_value(row).context("read the new row back")?;
        for (key, value) in fields.as_object().into_iter().flatten() {
            if let Some(items) = string_items(value).filter(|items| !items.is_empty()) {
                self.write_items(&path, key, &items)?;
            }
        }
        Ok(())
    }

    /// Put a section's rows in the order these ids give.
    ///
    /// # Errors
    ///
    /// Fails when an id is not in the section.
    pub fn reorder_rows(&mut self, name: &str, ids: &[String]) -> Result<()> {
        let current = self.ids(name);
        if current == ids {
            return Ok(());
        }
        let order = ids
            .iter()
            .map(|id| self.position_of(name, id).map(Segment::Index))
            .collect::<Result<Vec<_>>>()?;
        self.yaml
            .reorder(&section(name), &order)
            .with_context(|| format!("reorder {name}"))
    }

    /// Leave only `kept` in a row's `blocked_by`.
    ///
    /// # Errors
    ///
    /// Fails when no row in the section carries `row`.
    pub fn rewrite_blockers(&mut self, name: &str, row: &str, kept: &[String]) -> Result<()> {
        let path = section(name).index(self.position_of(name, row)?);
        self.write_items(&path, "blocked_by", kept)
            .with_context(|| format!("rewrite {row}'s blocked_by"))
    }

    fn take(&mut self, name: &str, index: usize) -> Result<yamled::Fragment> {
        self.yaml
            .take(&section(name).index(index))
            .with_context(|| format!("take {name}[{index}]"))
    }

    fn put(&mut self, name: &str, index: usize, row: &yamled::Fragment) -> Result<()> {
        self.yaml
            .put(&section(name), index, row)
            .with_context(|| format!("put a row into {name}"))
    }

    /// Add `name: []` at the end of the file when the ledger has no such list,
    /// which `horizon` is allowed to be.
    fn ensure_section(&mut self, name: &str) -> Result<()> {
        if self.yaml.node(&section(name)).is_some() {
            return Ok(());
        }
        self.yaml
            .insert(&Path::root(), name, &NO_ITEMS, Position::End)
            .with_context(|| format!("add {name}"))
    }
}

/// The strings of a list field, or `None` for any other value.
fn string_items(value: &serde_json::Value) -> Option<Vec<String>> {
    value
        .as_array()?
        .iter()
        .map(|item| item.as_str().map(ToOwned::to_owned))
        .collect()
}
