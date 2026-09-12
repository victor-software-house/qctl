use anyhow::{Context, Result};
use std::ops::Range;
use yaml_serde::Value;

pub(super) fn sequence_like(source: &str, span: &Range<usize>, value: &Value) -> Result<String> {
    let raw = sequence(value)?;
    let line_start = source[..span.start].rfind('\n').map_or(0, |at| at + 1);
    let line_prefix = &source[line_start..span.start];
    let existing_block = source[span.clone()].trim_start().starts_with('-');
    let indent = if existing_block {
        line_prefix.len()
    } else {
        line_prefix.len() - line_prefix.trim_start().len() + 2
    };
    let pad = " ".repeat(indent);
    let written = raw
        .lines()
        .map(|line| {
            if line.is_empty() {
                String::new()
            } else {
                format!("{pad}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    if existing_block {
        Ok(written.trim_start().to_owned())
    } else {
        Ok(format!("\n{written}"))
    }
}

pub(super) fn mapping_entry(value: &Value) -> Result<String> {
    let rendered = sequence(value)?
        .lines()
        .map(|line| {
            if line.is_empty() {
                String::new()
            } else {
                format!("  {line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(format!("notes:\n{rendered}"))
}

fn sequence(value: &Value) -> Result<String> {
    let Value::Sequence(items) = value else {
        return yaml_serde::to_string(value).context("render notes");
    };
    let mut rendered = String::new();
    for value in items {
        let Value::String(note) = value else {
            return yaml_serde::to_string(value).context("render notes");
        };
        if !rendered.is_empty() {
            rendered.push('\n');
        }
        rendered.push_str(&item(note)?);
    }
    Ok(rendered)
}

fn item(note: &str) -> Result<String> {
    let scalar = yaml_serde::to_string(&Value::String(note.to_owned()))?;
    let escaped_item = format!("- {}", serde_json::to_string(note)?);

    if note.contains('\n') {
        let trailing = note.len() - note.trim_end_matches('\n').len();
        let has_whitespace_only_line = note
            .split('\n')
            .any(|line| !line.is_empty() && line.trim().is_empty());
        if trailing > 1 || has_whitespace_only_line {
            return Ok(escaped_item.clone());
        }
        let content = &note[..note.len() - trailing];
        let mut lines = content.split('\n').collect::<Vec<_>>();
        lines.extend(std::iter::repeat_n("", trailing.saturating_sub(1)));
        let body = lines
            .into_iter()
            .map(|line| {
                if line.is_empty() {
                    String::new()
                } else {
                    format!("  {line}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let chomp = match trailing {
            0 => "-",
            1 => "",
            _ => "+",
        };
        let block = format!("- |2{chomp}\n{body}");
        return Ok(if round_trips(&block, note) {
            block
        } else {
            escaped_item.clone()
        });
    }

    let scalar = scalar.trim_end();
    if scalar.starts_with(['\'', '"']) {
        let block = format!("- >2-\n  {note}");
        return Ok(if round_trips(&block, note) {
            block
        } else {
            escaped_item
        });
    }
    Ok(format!("- {scalar}"))
}

fn round_trips(rendered: &str, expected: &str) -> bool {
    serde_yml::from_str::<Vec<String>>(rendered).is_ok_and(|notes| notes.as_slice() == [expected])
}
