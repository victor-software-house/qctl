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

use anyhow::{Context, Result, bail};
use std::ops::Range;
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

    /// Move a row to the head of its own section.
    pub fn move_to_front(&mut self, section: &str, id: &str) -> Result<()> {
        if self.ids(section)?.first().is_some_and(|first| first == id) {
            return Ok(());
        }
        let row = self.cut(section, id)?;
        self.paste_front(section, &row)
    }

    /// Take a row out of one section and put it at the head of another, with the
    /// keys it gains and loses on the way.
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

    /// Add a row to the end of a section, written from a value so the serializer
    /// decides the quoting rather than this file.
    pub fn append(&mut self, section: &str, value: &Value) -> Result<()> {
        let rendered = yaml_serde::to_string(value).context("render the new row")?;
        self.paste_back(section, &as_item(&rendered))
    }

    /// Put a section's rows in the order these ids name, keeping each row's
    /// text — and the comment above it — as it was written.
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

    /// Take an id out of every `blocked_by` that still names it. A blocker is
    /// resolved against the queue, so a row left naming an archived id fails
    /// `check` the moment the verb returns.
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

    /// The same list, shorter, in the style it was already written in.
    ///
    /// No quoting decision arises: a blocker is a task id, and the schema's
    /// pattern leaves no id that could need quotes. `Op::Replace` is not used
    /// here because it cannot write a non-empty sequence back into a mapping.
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

    /// Apply patch operations, stated against routes rather than offsets, so
    /// they cannot invalidate one another.
    fn patch(&mut self, patches: &[Patch]) -> Result<()> {
        let parsed = self.parsed()?;
        let edited = apply_yaml_patches(&parsed, patches).context("edit this ledger")?;
        edited.source().clone_into(&mut self.source);
        Ok(())
    }

    /// Remove a row and hand back its text, dedented to stand on its own. The
    /// blank lines that separated it from the row above go with it, so the seam
    /// left behind reads the way it did before the row was written.
    fn cut(&mut self, section: &str, id: &str) -> Result<String> {
        let index = self.position_of(section, id)?;
        let rows = self.rows(section)?;
        let span = rows[index].clone();
        let indent = self.indent_of(section)?;
        let row = dedent(self.source[span.clone()].trim_end(), indent);
        let last = index + 1 == rows.len();
        // A row's span reaches to where the next row begins, so it already holds
        // the blank line that separated them. Taking the blank line above as well
        // would remove two separators for one row, and the rows either side of it
        // would end up written against each other.
        //
        // The last row has no next row, so its span stops at the end of its text:
        // there the blank line above is the one to take, along with the line
        // ending, or the section keeps a blank line nobody wrote.
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

    /// Put a row at the head of a section, separated from what follows.
    fn paste_front(&mut self, section: &str, row: &str) -> Result<()> {
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

    /// Put a row after a section's last one.
    fn paste_back(&mut self, section: &str, row: &str) -> Result<()> {
        self.open(section)?;
        let indent = self.indent_of(section)?;
        let rows = self.rows(section)?;
        let at = match rows.last() {
            // Past the last row's line ending, so the blank line separating it
            // from the new row lands between them rather than after.
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

    /// Where each row of a section begins and ends, in file order.
    fn rows(&self, section: &str) -> Result<Vec<Range<usize>>> {
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

    /// Which row of a section carries this id.
    fn position_of(&self, section: &str, id: &str) -> Result<usize> {
        self.ids(section)?
            .iter()
            .position(|carried| carried == id)
            .with_context(|| format!("{id} is not in {section}"))
    }

    /// The start of the line an offset sits on, widened over the comment lines
    /// written directly above it.
    fn line_holding(&self, offset: usize) -> usize {
        let mut start = self.source[..offset].rfind('\n').map_or(0, |at| at + 1);
        while let Some(above) = self.source[..start.saturating_sub(1)].rfind('\n') {
            if !self.source[above + 1..start].trim_start().starts_with('#') {
                break;
            }
            start = above + 1;
        }
        start
    }

    /// The beginning of the run of blank lines ending at this offset.
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

    /// An end offset pulled back over trailing blank and comment lines, which
    /// belong to the file rather than to the row that happens to precede them.
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

    /// How deep this section's rows sit, taken from the rows already there.
    fn indent_of(&self, section: &str) -> Result<usize> {
        let Some(first) = self.rows(section)?.first().cloned() else {
            return Ok(DEFAULT_INDENT);
        };
        let line = &self.source[first.start..];
        Ok(line.len() - line.trim_start().len())
    }

    /// Where a section's first row goes: just past its key line.
    fn front_of(&self, section: &str) -> Result<usize> {
        let after = self.key_span(section)?.1;
        Ok(self.source[after..]
            .find('\n')
            .map_or(self.source.len(), |at| after + at + 1))
    }

    /// Turn `section: []` into `section:`, so a first row has a block to join.
    fn open(&mut self, section: &str) -> Result<()> {
        let (from, line_end) = self.key_line(section)?;
        if !self.source[from..line_end].trim_end().ends_with("[]") {
            return Ok(());
        }
        self.source
            .replace_range(from..line_end, &format!("{section}:"));
        Ok(())
    }

    /// Turn an emptied `section:` back into `section: []`.
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

    /// The whole line a section's key is written on.
    fn key_line(&self, section: &str) -> Result<(usize, usize)> {
        let (from, key_end) = self.key_span(section)?;
        let line_end = self.source[key_end..]
            .find('\n')
            .map_or(self.source.len(), |at| key_end + at);
        Ok((from, line_end))
    }
}

/// Drop keys from one row and add others, with the row standing alone so no
/// removal can reach a neighbour's comment.
fn revise(row: &str, drop: &[&str], add: &[(&str, Value)]) -> Result<String> {
    let document = Parsed::new(row.to_owned()).context("read the row on its own")?;
    let patches: Vec<Patch> = drop
        .iter()
        .filter(|key| document.query_exists(&route!(0, **key)))
        .map(|key| Patch {
            route: route!(0, *key),
            operation: Op::Remove,
        })
        .collect();
    let mut kept = if patches.is_empty() {
        row.to_owned()
    } else {
        apply_yaml_patches(&document, &patches)
            .context("revise the row")?
            .source()
            .to_owned()
    };
    // Added keys are appended as their own lines rather than patched in, so a
    // list arrives written the way the ledgers write lists. The values are still
    // the serializer's: only where the text sits is decided here.
    for (key, value) in add {
        kept = format!(
            "{}\n{}",
            kept.trim_end(),
            indent_by(&render(key, value)?, 2)
        );
    }
    Ok(kept.trim_end().to_owned())
}

/// One key and its value, as the serializer writes them.
fn render(key: &str, value: &Value) -> Result<String> {
    let mut mapping = yaml_serde::Mapping::new();
    mapping.insert(Value::from(key), value.clone());
    let rendered =
        yaml_serde::to_string(&Value::Mapping(mapping)).with_context(|| format!("render {key}"))?;
    Ok(as_the_ledgers_write_lists(&rendered))
}

/// A rendered mapping turned into a block sequence item.
fn as_item(mapping: &str) -> String {
    let written = as_the_ledgers_write_lists(mapping);
    let mut lines = written.lines();
    let Some(first) = lines.next() else {
        return String::new();
    };
    let rest = lines.fold(String::new(), |mut item, line| {
        item.push_str("\n  ");
        item.push_str(line);
        item
    });
    format!("- {first}{rest}")
}

/// The serializer puts a nested list flush against its key. Every ledger indents
/// it, so a generated row is written the way the rows around it are.
fn as_the_ledgers_write_lists(rendered: &str) -> String {
    rendered
        .trim_end()
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("- ") {
                format!("  {line}")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The id a row's text declares.
fn id_of(text: &str) -> Option<&str> {
    text.lines().find_map(|line| {
        line.trim_start()
            .trim_start_matches("- ")
            .strip_prefix("id:")
            .map(|value| value.trim().trim_matches(['"', '\'']))
    })
}

fn dedent(text: &str, spaces: usize) -> String {
    text.lines()
        .map(|line| line.get(spaces..).unwrap_or(line.trim_start()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn indent_by(text: &str, spaces: usize) -> String {
    let pad = " ".repeat(spaces);
    text.lines()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                format!("{pad}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Fail rather than write something the next verb cannot read.
pub fn must_still_parse(source: &str) -> Result<()> {
    if serde_yml::from_str::<serde_yml::Value>(source).is_err() {
        bail!("this edit would leave a file that is not valid YAML");
    }
    Ok(())
}
