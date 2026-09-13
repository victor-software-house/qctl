use crate::cli::{EditArgs, RemoveField};
use anyhow::{Context, Result, ensure};
use std::collections::HashSet;

pub(super) struct Removals<'a> {
    note: Vec<&'a str>,
    acceptance: Vec<&'a str>,
    link: Vec<&'a str>,
    blocked_by: Vec<&'a str>,
    evidence: Vec<&'a str>,
}

impl<'a> Removals<'a> {
    pub(super) fn new(args: &'a EditArgs) -> Self {
        let mut grouped = Self {
            note: Vec::new(),
            acceptance: Vec::new(),
            link: Vec::new(),
            blocked_by: Vec::new(),
            evidence: Vec::new(),
        };
        for removal in &args.remove {
            grouped.for_field_mut(removal.field).push(&removal.selector);
        }
        grouped
    }

    pub(super) fn for_field(&self, field: RemoveField) -> &[&'a str] {
        match field {
            RemoveField::Note => &self.note,
            RemoveField::Acceptance => &self.acceptance,
            RemoveField::Link => &self.link,
            RemoveField::BlockedBy => &self.blocked_by,
            RemoveField::Evidence => &self.evidence,
        }
    }

    fn for_field_mut(&mut self, field: RemoveField) -> &mut Vec<&'a str> {
        match field {
            RemoveField::Note => &mut self.note,
            RemoveField::Acceptance => &mut self.acceptance,
            RemoveField::Link => &mut self.link,
            RemoveField::BlockedBy => &mut self.blocked_by,
            RemoveField::Evidence => &mut self.evidence,
        }
    }
}

pub(super) fn apply(
    current: &[String],
    additions: &[String],
    removals: &[&str],
    reorder: Option<&str>,
) -> Result<Vec<String>> {
    let mut list = current.to_vec();
    for selector in removals {
        remove(&mut list, selector)?;
    }
    if let Some(order) = reorder {
        list = permute(&list, order)?;
    }
    for item in additions {
        ensure!(!item.is_empty(), "list item is empty");
        ensure!(!list.contains(item), "duplicate {item}");
        list.push(item.clone());
    }
    Ok(list)
}

fn remove(list: &mut Vec<String>, selector: &str) -> Result<()> {
    if let Ok(index) = selector.parse::<usize>()
        && index >= 1
        && index <= list.len()
    {
        list.remove(index - 1);
        return Ok(());
    }
    let index = list
        .iter()
        .position(|item| item == selector)
        .with_context(|| format!("{selector} is not in the list"))?;
    list.remove(index);
    Ok(())
}

fn permute(list: &[String], order: &str) -> Result<Vec<String>> {
    let indexes: Result<Vec<usize>, _> = order
        .split(',')
        .map(|part| part.trim().parse::<usize>())
        .collect();
    let indexes = indexes.context("--reorder-notes needs a permutation such as 3,1,2")?;
    ensure!(
        indexes.len() == list.len(),
        "--reorder-notes must name every note once"
    );
    let mut seen = HashSet::new();
    let mut reordered = Vec::with_capacity(list.len());
    for index in indexes {
        ensure!(
            index >= 1 && index <= list.len(),
            "--reorder-notes index {index} is out of range"
        );
        ensure!(seen.insert(index), "--reorder-notes repeats {index}");
        reordered.push(list[index - 1].clone());
    }
    Ok(reordered)
}
