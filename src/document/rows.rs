//! Rows found by id, moved between and within sections.
//!
//! A row moves with the comments written directly above it. A list that loses
//! its last row becomes `[]`, and a row put into `[]` opens it as a block list.
//! yamled owns all three rules.

use super::{Change, Document, NO_ITEMS, section};
use anyhow::{Context, Result};
use serde::Serialize;
use yamled::{Path, Position, Segment};

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

    pub fn position_of(&self, name: &str, id: &str) -> Result<usize> {
        self.ids(name)
            .iter()
            .position(|carried| carried == id)
            .with_context(|| format!("{id} is not in {name}"))
    }

    pub fn move_to_front(&mut self, name: &str, id: &str) -> Result<()> {
        self.move_row(name, id, 0)
    }

    /// Move a row within its section so it ends up at `destination`.
    pub fn move_row(&mut self, name: &str, id: &str, destination: usize) -> Result<()> {
        let from = self.position_of(name, id)?;
        if from == destination {
            return Ok(());
        }
        let row = self.take(name, from)?;
        self.put(name, destination, &row)
    }

    /// Move a row to the front of another section, then change its fields.
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

    /// Add a row at the end of a section, then its notes, written one by one
    /// in the style each note gets. `row` carries no notes of its own.
    pub fn append<T: Serialize>(&mut self, name: &str, row: &T, notes: &[String]) -> Result<()> {
        self.ensure_section(name)?;
        let list = section(name);
        self.yaml.push(&list, row).context("add the row")?;
        if notes.is_empty() {
            return Ok(());
        }
        let row = list.index(self.ids(name).len() - 1);
        self.yaml
            .insert(&row, "notes", &NO_ITEMS, Position::End)
            .context("add notes")?;
        self.push_notes(&row.key("notes"), notes)
    }

    /// Put a section's rows in the order these ids give.
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
    pub fn rewrite_blockers(&mut self, name: &str, row: &str, kept: &[String]) -> Result<()> {
        let path = section(name)
            .index(self.position_of(name, row)?)
            .key("blocked_by");
        self.yaml
            .replace(&path, kept)
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
