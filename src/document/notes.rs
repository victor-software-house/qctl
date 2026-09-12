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
    if note.contains('\n') {
        let body = note
            .split('\n')
            .map(|line| {
                if line.is_empty() {
                    String::new()
                } else {
                    format!("  {line}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        return Ok(format!("- |2-\n{body}"));
    }

    let scalar = yaml_serde::to_string(&Value::String(note.to_owned()))?;
    let scalar = scalar.trim_end();
    if scalar.starts_with(['\'', '"']) {
        return Ok(format!("- >2-\n  {note}"));
    }
    Ok(format!("- {scalar}"))
}
