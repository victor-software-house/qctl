use super::notes;
use anyhow::{Context, Result};
use std::ops::Range;
use yaml_serde::Value;

pub(super) fn sequence_like(source: &str, span: &Range<usize>, value: &Value) -> Result<String> {
    if let Some(rendered) = block_scalar_sequence(source, span, value) {
        return Ok(rendered);
    }
    if source[span.clone()].trim_start().starts_with('[') {
        let Value::Sequence(items) = value else {
            unreachable!("called only for a sequence")
        };
        let rendered = items
            .iter()
            .map(|item| yaml_serde::to_string(item).context("render a flow-sequence item"))
            .collect::<Result<Vec<_>>>()?;
        if rendered.iter().all(|item| !item.trim_end().contains('\n')) {
            return Ok(format!(
                "[{}]",
                rendered
                    .iter()
                    .map(|item| item.trim_end())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        return serde_json::to_string(value).context("render a flow sequence");
    }

    let rendered = yaml_serde::to_string(value).context("render a block sequence")?;
    let line_start = source[..span.start].rfind('\n').map_or(0, |at| at + 1);
    let line_prefix = &source[line_start..span.start];
    let existing_block = source[span.clone()].trim_start().starts_with('-');
    let indent = if existing_block {
        line_prefix.len()
    } else {
        line_prefix.len() - line_prefix.trim_start().len() + 2
    };
    let pad = " ".repeat(indent);
    let written = rendered
        .trim_end()
        .lines()
        .map(|line| format!("{pad}{line}"))
        .collect::<Vec<_>>()
        .join("\n");
    if existing_block {
        Ok(written.trim_start().to_owned())
    } else {
        Ok(format!("\n{written}"))
    }
}

fn block_scalar_sequence(source: &str, span: &Range<usize>, value: &Value) -> Option<String> {
    let Value::Sequence(items) = value else {
        return None;
    };
    let mut lines = source[span.clone()].trim_start().lines();
    let indicator = lines.next()?.trim();
    if !matches!(indicator.chars().next(), Some('>' | '|')) {
        return None;
    }
    let body: Vec<&str> = lines.collect();
    let depth = body
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()?;
    let paragraphs: Vec<Vec<&str>> = body
        .split(|line| line.trim().is_empty())
        .filter(|paragraph| !paragraph.is_empty())
        .map(<[&str]>::to_vec)
        .collect();
    if paragraphs.len() != items.len() {
        return None;
    }

    let line_start = source[..span.start].rfind('\n').map_or(0, |at| at + 1);
    let key_indent =
        source[line_start..span.start].len() - source[line_start..span.start].trim_start().len();
    let item_pad = " ".repeat(key_indent + 2);
    let content_pad = " ".repeat(key_indent + 4);
    let mut rendered = String::new();
    for paragraph in paragraphs {
        rendered.push('\n');
        rendered.push_str(&item_pad);
        rendered.push_str("- ");
        rendered.push_str(indicator);
        for line in paragraph {
            rendered.push('\n');
            rendered.push_str(&content_pad);
            rendered.push_str(line.get(depth..).unwrap_or(line.trim_start()));
        }
    }
    Some(rendered)
}

pub(super) fn mapping_entry(key: &str, value: &Value) -> Result<String> {
    if key == "notes" {
        return notes::mapping_entry(value);
    }
    let mut mapping = yaml_serde::Mapping::new();
    mapping.insert(Value::from(key), value.clone());
    let rendered =
        yaml_serde::to_string(&Value::Mapping(mapping)).with_context(|| format!("render {key}"))?;
    Ok(indent_nested_lists(&rendered))
}

pub(super) fn sequence_item(mapping: &str, value: &Value) -> Result<String> {
    let notes = value
        .as_mapping()
        .and_then(|mapping| mapping.get(Value::from("notes")))
        .filter(|value| matches!(value, Value::Sequence(_)));
    let without_notes = if notes.is_some() {
        let mut value = value.clone();
        value
            .as_mapping_mut()
            .context("a rendered row must be a mapping")?
            .remove(Value::from("notes"));
        yaml_serde::to_string(&value).context("render the row without notes")?
    } else {
        mapping.to_owned()
    };
    let written = indent_nested_lists(&without_notes);
    let mut lines = written.lines();
    let Some(first) = lines.next() else {
        return Ok(String::new());
    };
    let rest = lines.fold(String::new(), |mut item, line| {
        item.push_str("\n  ");
        item.push_str(line);
        item
    });
    let mut row = format!("- {first}{rest}");
    if let Some(notes) = notes {
        row.push('\n');
        row.push_str(
            &notes::mapping_entry(notes)?
                .lines()
                .map(|line| format!("  {line}"))
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }
    Ok(row)
}

fn indent_nested_lists(rendered: &str) -> String {
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
