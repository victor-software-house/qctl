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
    for item_value in items {
        let Value::String(note) = item_value else {
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
        let body = content
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
        let chomp = if trailing == 0 { "-" } else { "" };
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

#[cfg(test)]
mod tests {
    use super::{item, sequence};
    use yaml_serde::Value;

    #[test]
    fn non_string_item_falls_back_to_the_whole_sequence() {
        let value = Value::Sequence(vec![
            Value::from("first"),
            Value::from(7),
            Value::from("last"),
        ]);
        let rendered = sequence(&value).expect("render sequence");
        let parsed: Value = serde_yml::from_str(&rendered).expect("parse sequence");
        assert_eq!(parsed, value);
    }

    #[test]
    fn multiple_trailing_newlines_use_exact_escaped_fallback() {
        let rendered = item("trailing\n\n").expect("render item");
        assert_eq!(rendered, r#"- "trailing\n\n""#);
        assert!(!rendered.contains("|2+"));
    }
}
