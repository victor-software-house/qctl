//! Lossless row traversal, movement, and section insertion.
//!
//! A row carries a directly attached leading comment. Cutting the last row also
//! takes its preceding blank separator; other rows carry the separator below
//! them. These rules keep moves byte-bounded and prevent one row from stealing a
//! neighbour's comment.

use super::text::{dedent, id_of, indent_by};
use super::value;
use super::{DEFAULT_INDENT, Document, revise};
use anyhow::{Context, Result};
use std::fmt::Write;
use std::ops::Range;
use yaml_serde::Value;
use yamlpath::{Route, route};

impl Document {
    pub fn ids(&self, section: &str) -> Result<Vec<String>> {
        let parsed = self.parsed()?;
        let mut ids = Vec::new();
        let mut index = 0;
        while let Ok(Some(feature)) = parsed.query_exact(&route!(section, index)) {
            if let Some(id) = id_of(parsed.extract(&feature)) {
                ids.push(id.to_owned());
            }
            index += 1;
        }
        Ok(ids)
    }

    pub fn move_to_front(&mut self, section: &str, id: &str) -> Result<()> {
        if self.ids(section)?.first().is_some_and(|first| first == id) {
            return Ok(());
        }
        let row = self.cut(section, id)?;
        self.paste_front(section, &row)
    }

    pub fn move_between(
        &mut self,
        from: &str,
        to: &str,
        id: &str,
        drop: &[&str],
        add: &[(&str, Value)],
    ) -> Result<()> {
        let row = self.cut(from, id)?;
        let row = revise(&row, drop, add)?;
        self.close_if_empty(from)?;
        self.paste_front(to, &row)
    }

    pub fn move_to_end(
        &mut self,
        from: &str,
        to: &str,
        id: &str,
        drop: &[&str],
        add: &[(&str, Value)],
    ) -> Result<()> {
        let row = self.cut(from, id)?;
        let row = revise(&row, drop, add)?;
        self.close_if_empty(from)?;
        self.paste_back(to, &row)
    }

    pub fn append(&mut self, section: &str, value: &Value) -> Result<()> {
        let rendered = yaml_serde::to_string(value).context("render the new row")?;
        self.paste_back(section, &value::sequence_item(&rendered, value)?)
    }

    pub fn reorder_rows(&mut self, section: &str, ids: &[String]) -> Result<()> {
        if self.ids(section)? == ids {
            return Ok(());
        }
        let mut rows = Vec::with_capacity(ids.len());
        for id in ids {
            rows.push(self.cut(section, id)?);
        }
        for row in rows {
            self.paste_back(section, &row)?;
        }
        Ok(())
    }

    /// Take an id out of every `blocked_by` that still names it.
    pub fn rewrite_blockers(&mut self, section: &str, row: &str, kept: &[String]) -> Result<()> {
        let index = self.position_of(section, row)?;
        let route: Route = route!(section, index, "blocked_by");
        let span = {
            let parsed = self.parsed()?;
            let feature = parsed
                .query_exact(&route)?
                .with_context(|| format!("{row} has no blocked_by to rewrite"))?;
            let (from, to) = feature.location.byte_span;
            from..to
        };
        let written = self.rewrite_list(&span, kept);
        self.source.replace_range(span, &written);
        Ok(())
    }

    fn rewrite_list(&self, span: &Range<usize>, kept: &[String]) -> String {
        let was = &self.source[span.clone()];
        if !was.trim_start().starts_with('-') {
            return format!("[{}]", kept.join(", "));
        }
        if kept.is_empty() {
            return "[]".to_owned();
        }
        let indent = self.source[..span.start]
            .rfind('\n')
            .map_or(0, |line| span.start - line - 1);
        let pad = " ".repeat(indent);
        kept.iter()
            .enumerate()
            .map(|(at, blocker)| {
                if at == 0 {
                    format!("- {blocker}")
                } else {
                    format!("\n{pad}- {blocker}")
                }
            })
            .collect()
    }

    /// Remove a row and return it dedented, without disturbing adjacent rows.
    pub fn cut(&mut self, section: &str, id: &str) -> Result<String> {
        let index = self.position_of(section, id)?;
        let rows = self.rows(section)?;
        let span = rows[index].clone();
        let indent = self.indent_of(section)?;
        let row = dedent(self.source[span.clone()].trim_end(), indent);
        let last = index + 1 == rows.len();
        let from = if last {
            self.start_of_blank_run(span.start)
        } else {
            span.start
        };
        let to = if last && self.source[span.end..].starts_with('\n') {
            span.end + 1
        } else {
            span.end
        };
        self.source.replace_range(from..to, "");
        Ok(row)
    }

    fn paste_front(&mut self, section: &str, row: &str) -> Result<()> {
        self.ensure_section(section)?;
        self.open(section)?;
        let crowded = !self.ids(section)?.is_empty();
        let indent = self.indent_of(section)?;
        let at = self.front_of(section)?;
        let block = if crowded {
            format!("{}\n\n", indent_by(row, indent))
        } else {
            format!("{}\n", indent_by(row, indent))
        };
        self.source.insert_str(at, &block);
        Ok(())
    }

    fn paste_back(&mut self, section: &str, row: &str) -> Result<()> {
        self.ensure_section(section)?;
        self.open(section)?;
        let indent = self.indent_of(section)?;
        let rows = self.rows(section)?;
        let at = match rows.last() {
            Some(last) => self.source[last.end..]
                .find('\n')
                .map_or(self.source.len(), |at| last.end + at + 1),
            None => self.front_of(section)?,
        };
        let block = if rows.is_empty() {
            format!("{}\n", indent_by(row, indent))
        } else {
            format!("\n{}\n", indent_by(row, indent))
        };
        self.source.insert_str(at, &block);
        Ok(())
    }

    pub(super) fn rows(&self, section: &str) -> Result<Vec<Range<usize>>> {
        let parsed = self.parsed()?;
        let mut bounds = Vec::new();
        let mut index = 0;
        while let Ok(Some(feature)) = parsed.query_exact(&route!(section, index)) {
            let (content, end) = feature.location.byte_span;
            bounds.push((self.line_holding(content), end));
            index += 1;
        }
        let last = bounds
            .last()
            .map(|(_, end)| self.without_trailing_remarks(*end));
        Ok(bounds
            .iter()
            .enumerate()
            .map(|(at, (start, _))| {
                let end = bounds
                    .get(at + 1)
                    .map_or(last.unwrap_or(*start), |(next, _)| *next);
                *start..end
            })
            .collect())
    }

    pub fn position_of(&self, section: &str, id: &str) -> Result<usize> {
        self.ids(section)?
            .iter()
            .position(|carried| carried == id)
            .with_context(|| format!("{id} is not in {section}"))
    }

    pub(super) fn line_holding(&self, offset: usize) -> usize {
        let mut start = self.source[..offset].rfind('\n').map_or(0, |at| at + 1);
        while let Some(above) = self.source[..start.saturating_sub(1)].rfind('\n') {
            if !self.source[above + 1..start].trim_start().starts_with('#') {
                break;
            }
            start = above + 1;
        }
        start
    }

    fn start_of_blank_run(&self, offset: usize) -> usize {
        let mut start = offset;
        while start > 0 {
            let line_start = self.source[..start - 1].rfind('\n').map_or(0, |at| at + 1);
            if !self.source[line_start..start - 1].trim().is_empty() {
                return start;
            }
            start = line_start;
        }
        start
    }

    fn without_trailing_remarks(&self, end: usize) -> usize {
        let mut end = end;
        loop {
            let line_start = self.source[..end].rfind('\n').map_or(0, |at| at + 1);
            let line = self.source[line_start..end].trim_start();
            if line_start == 0 || !(line.is_empty() || line.starts_with('#')) {
                return end;
            }
            end = line_start.saturating_sub(1);
        }
    }

    fn indent_of(&self, section: &str) -> Result<usize> {
        let Some(first) = self.rows(section)?.first().cloned() else {
            return Ok(DEFAULT_INDENT);
        };
        let line = &self.source[first.start..];
        Ok(line.len() - line.trim_start().len())
    }

    fn front_of(&self, section: &str) -> Result<usize> {
        let after = self.key_span(section)?.1;
        Ok(self.source[after..]
            .find('\n')
            .map_or(self.source.len(), |at| after + at + 1))
    }

    fn open(&mut self, section: &str) -> Result<()> {
        let (from, line_end) = self.key_line(section)?;
        if !self.source[from..line_end].trim_end().ends_with("[]") {
            return Ok(());
        }
        self.source
            .replace_range(from..line_end, &format!("{section}:"));
        Ok(())
    }

    fn ensure_section(&mut self, section: &str) -> Result<()> {
        if self.parsed()?.query_key_only(&route!(section)).is_ok() {
            return Ok(());
        }
        if !self.source.ends_with('\n') {
            self.source.push('\n');
        }
        writeln!(self.source, "{section}: []").context("write the missing list key")?;
        Ok(())
    }

    fn close_if_empty(&mut self, section: &str) -> Result<()> {
        if !self.ids(section)?.is_empty() {
            return Ok(());
        }
        let (from, line_end) = self.key_line(section)?;
        self.source
            .replace_range(from..line_end, &format!("{section}: []"));
        Ok(())
    }

    fn key_span(&self, section: &str) -> Result<(usize, usize)> {
        let parsed = self.parsed()?;
        let key = parsed
            .query_key_only(&route!(section))
            .with_context(|| format!("{section}: is missing"))?;
        Ok(key.location.byte_span)
    }

    pub(super) fn key_line(&self, section: &str) -> Result<(usize, usize)> {
        let (from, key_end) = self.key_span(section)?;
        let line_end = self.source[key_end..]
            .find('\n')
            .map_or(self.source.len(), |at| key_end + at);
        Ok((from, line_end))
    }

    pub fn paste_at(&mut self, section: &str, index: usize, row: &str) -> Result<()> {
        let count = self.ids(section)?.len();
        if index == 0 {
            return self.paste_front(section, row);
        }
        if index >= count {
            return self.paste_back(section, row);
        }
        self.ensure_section(section)?;
        self.open(section)?;
        let indent = self.indent_of(section)?;
        let rows = self.rows(section)?;
        let at = rows[index].start;
        let block = format!("{}\n\n", indent_by(row, indent));
        self.source.insert_str(at, &block);
        Ok(())
    }
}
