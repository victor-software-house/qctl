use crate::cli::{EditArgs, UnsetField};
use crate::document::Change;
use anyhow::{Result, bail, ensure};
use std::path::Path;

pub(super) struct ListEdit<'a> {
    key: &'static str,
    before: &'a [String],
    after: Vec<String>,
}

impl<'a> ListEdit<'a> {
    pub(super) fn new(key: &'static str, before: &'a [String], after: Vec<String>) -> Self {
        Self { key, before, after }
    }
}

pub(super) fn changes(args: &EditArgs, path: &Path, lists: &[ListEdit<'_>]) -> Result<Vec<Change>> {
    if args.patch.is_some() && args.unset.contains(&UnsetField::Patch) {
        bail!("use --patch or --unset patch, not both");
    }
    if args.plan.is_some() && args.unset.contains(&UnsetField::Plan) {
        bail!("use --plan or --unset plan, not both");
    }
    crate::mutate::require_plan(path, args.plan.as_deref())?;

    let mut changes = Vec::new();
    push_scalar(&mut changes, "title", args.title.as_deref())?;
    push_scalar(&mut changes, "scope", args.scope.as_deref())?;
    push_scalar(&mut changes, "outcome", args.outcome.as_deref())?;
    if let Some(kind) = args.kind {
        changes.push(Change::set("kind", kind.to_string()));
    }
    push_scalar(&mut changes, "open", args.open.as_deref())?;
    push_optional(
        &mut changes,
        "patch",
        args.patch.as_deref(),
        args.unset.contains(&UnsetField::Patch),
    )?;
    push_optional(
        &mut changes,
        "plan",
        args.plan.as_deref(),
        args.unset.contains(&UnsetField::Plan),
    )?;
    for list in lists {
        push_list(&mut changes, list);
    }
    Ok(changes)
}

fn push_scalar(changes: &mut Vec<Change>, key: &str, value: Option<&str>) -> Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    ensure!(!value.is_empty(), "--{key} needs a value");
    changes.push(Change::set(key, value));
    Ok(())
}

fn push_optional(
    changes: &mut Vec<Change>,
    key: &str,
    value: Option<&str>,
    unset: bool,
) -> Result<()> {
    if unset {
        changes.push(Change::unset(key));
        return Ok(());
    }
    push_scalar(changes, key, value)
}

fn push_list(changes: &mut Vec<Change>, list: &ListEdit<'_>) {
    if list.before == list.after.as_slice() {
        return;
    }
    changes.push(Change::List {
        key: list.key.to_owned(),
        before: list.before.to_vec(),
        after: list.after.clone(),
    });
}
