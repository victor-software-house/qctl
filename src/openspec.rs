//! The `openspec/` folder next to a ledger: which changes are open, where each
//! archived one went, and which change a row's `plan:` names.
//!
//! Only directory names are read, never a file's contents, so an `openspec`
//! release that changes what a spec says leaves this module alone. A row names
//! a change through its `plan:` path and nothing else: a change mentioned only
//! in a note is not named.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::schema::Ledger;

/// Where `openspec` keeps its changes, relative to the ledger's directory.
const CHANGES: &str = "openspec/changes";

/// The folder `openspec archive` moves a finished change into.
const ARCHIVE: &str = "archive";

/// The change a `plan:` path names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Named<'a> {
    /// A file inside `openspec/changes/<name>/`.
    Open {
        /// The change.
        name: &'a str,
        /// The path inside the change folder, such as `proposal.md`.
        file: &'a str,
    },
    /// A file inside `openspec/changes/archive/<YYYY-MM-DD>-<name>/`.
    Archived {
        /// The change.
        name: &'a str,
    },
}

impl<'a> Named<'a> {
    /// The change's name, open or archived.
    #[must_use]
    pub fn name(self) -> &'a str {
        match self {
            Self::Open { name, .. } | Self::Archived { name } => name,
        }
    }
}

/// The change `plan` names, if it names one.
#[must_use]
pub fn named(plan: &str) -> Option<Named<'_>> {
    let rest = plan.strip_prefix(CHANGES)?.strip_prefix('/')?;
    let (folder, file) = rest.split_once('/')?;
    if file.is_empty() {
        return None;
    }
    if folder == ARCHIVE {
        let (folder, file) = file.split_once('/')?;
        if file.is_empty() {
            return None;
        }
        return archived_name(folder).map(|name| Named::Archived { name });
    }
    Some(Named::Open { name: folder, file })
}

/// `<name>` from an archive folder called `<YYYY-MM-DD>-<name>`.
fn archived_name(folder: &str) -> Option<&str> {
    let date = folder.as_bytes().get(..10)?;
    let dated = date.iter().enumerate().all(|(at, byte)| match at {
        4 | 7 => *byte == b'-',
        _ => byte.is_ascii_digit(),
    });
    let name = folder.get(10..)?.strip_prefix('-')?;
    (dated && !name.is_empty()).then_some(name)
}

/// The changes under one `openspec/` folder.
#[derive(Debug, Default)]
pub struct Layout {
    open: BTreeSet<String>,
    /// Each archived change's folders, as paths relative to the ledger's
    /// directory. More than one means the name was reused.
    archived: BTreeMap<String, Vec<String>>,
}

impl Layout {
    /// The layout next to `ledger`, or `None` when no `openspec/` folder sits
    /// beside it, in which case no agreement rule applies.
    ///
    /// # Errors
    ///
    /// Fails when a folder that exists cannot be listed.
    pub fn read(ledger: &Path) -> Result<Option<Self>> {
        let root = ledger.parent().unwrap_or(Path::new("."));
        if !root.join("openspec").is_dir() {
            return Ok(None);
        }
        let changes = root.join(CHANGES);
        let mut layout = Self::default();
        for name in folders(&changes)? {
            if name != ARCHIVE {
                layout.open.insert(name);
            }
        }
        for folder in folders(&changes.join(ARCHIVE))? {
            if let Some(name) = archived_name(&folder) {
                layout
                    .archived
                    .entry(name.to_owned())
                    .or_default()
                    .push(format!("{CHANGES}/{ARCHIVE}/{folder}"));
            }
        }
        Ok(Some(layout))
    }

    /// Whether `openspec/changes/<name>/` exists.
    #[must_use]
    pub fn is_open(&self, name: &str) -> bool {
        self.open.contains(name)
    }

    /// Every open change, by name.
    pub fn open(&self) -> impl Iterator<Item = &str> {
        self.open.iter().map(String::as_str)
    }

    /// The archive folders a change was moved into, oldest first.
    #[must_use]
    pub fn archive_folders(&self, name: &str) -> &[String] {
        self.archived.get(name).map_or(&[], Vec::as_slice)
    }
}

/// Folder names directly under `dir`, sorted; none when `dir` does not exist.
fn folders(dir: &Path) -> Result<Vec<String>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("list {}", dir.display()))? {
        let entry = entry.with_context(|| format!("list {}", dir.display()))?;
        if entry.file_type()?.is_dir()
            && let Some(name) = entry.file_name().to_str()
        {
            names.push(name.to_owned());
        }
    }
    names.sort_unstable();
    Ok(names)
}

/// Where a plan that points into an open change went when that change was
/// archived: the same file name inside its one archive folder.
#[derive(Debug, Eq, PartialEq)]
pub enum Moved {
    /// One archive folder holds the file.
    To {
        /// The archive folder, relative to the ledger's directory.
        folder: String,
        /// The plan path inside it.
        plan: String,
    },
    /// Several archive folders end in the change's name.
    Ambiguous(Vec<String>),
}

/// Where `plan` went, when it points into an open change whose file is
/// missing and an archive folder of that change exists.
#[must_use]
pub fn moved(layout: &Layout, root: &Path, plan: &str) -> Option<Moved> {
    let Some(Named::Open { name, file }) = named(plan) else {
        return None;
    };
    if root.join(plan).is_file() {
        return None;
    }
    match layout.archive_folders(name) {
        [] => None,
        [folder] => {
            let target = format!("{folder}/{file}");
            root.join(&target).is_file().then(|| Moved::To {
                folder: folder.clone(),
                plan: target,
            })
        }
        several => Some(Moved::Ambiguous(several.to_vec())),
    }
}

/// The open change queued row `id` would leave behind if it were archived now:
/// its plan names an open change and no other queued or horizon row names it.
#[must_use]
pub fn left_open<'a>(ledger: &'a Ledger, layout: &Layout, id: &str) -> Option<&'a str> {
    let task = ledger.queue.iter().find(|task| task.id == id)?;
    let Some(Named::Open { name, .. }) = task.plan.as_deref().and_then(named) else {
        return None;
    };
    (layout.is_open(name) && !still_wanted(ledger, name, Some(id))).then_some(name)
}

/// Whether a queued or horizon row other than `except` names open change
/// `name`.
fn still_wanted(ledger: &Ledger, name: &str, except: Option<&str>) -> bool {
    let names = |plan: Option<&str>| matches!(plan.and_then(named), Some(Named::Open { name: named, .. }) if named == name);
    ledger
        .queue
        .iter()
        .filter(|task| Some(task.id.as_str()) != except)
        .any(|task| names(task.plan.as_deref()))
        || ledger
            .horizon
            .iter()
            .filter(|task| Some(task.id.as_str()) != except)
            .any(|task| names(task.plan.as_deref()))
}

/// Every disagreement between the ledger's rows and the changes in `layout`
/// that `check` reports on its own: a finished row whose change is still open,
/// and an open change no row names. A plan pointing into a change since
/// archived is reported with the missing plan, by [`moved`].
#[must_use]
pub fn problems(ledger: &Ledger, layout: &Layout) -> Vec<String> {
    let mut problems = Vec::new();
    for task in &ledger.archive {
        let Some(Named::Open { name, .. }) = task.plan.as_deref().and_then(named) else {
            continue;
        };
        if layout.is_open(name)
            && !still_wanted(ledger, name, None)
            && task.keep_change_open.is_none()
        {
            problems.push(format!(
                "{} is done but its change {name} is still open; archive the change with openspec archive and run qctl close-from-openspec, or set keep_change_open with a reason",
                task.id
            ));
        }
    }
    let named_open: BTreeSet<&str> = ledger
        .queue
        .iter()
        .map(|task| task.plan.as_deref())
        .chain(ledger.archive.iter().map(|task| task.plan.as_deref()))
        .chain(ledger.horizon.iter().map(|task| task.plan.as_deref()))
        .filter_map(|plan| match plan.and_then(named) {
            Some(Named::Open { name, .. }) => Some(name),
            _ => None,
        })
        .collect();
    for name in layout.open() {
        if !named_open.contains(name) {
            problems.push(format!(
                "change {name} is named by no row; point a row's plan at {CHANGES}/{name}/proposal.md"
            ));
        }
    }
    problems
}

#[cfg(test)]
mod tests {
    use super::{Named, archived_name, named};

    #[test]
    fn an_open_change_is_named_by_its_folder() {
        assert_eq!(
            named("openspec/changes/add-thing/proposal.md"),
            Some(Named::Open {
                name: "add-thing",
                file: "proposal.md"
            })
        );
    }

    #[test]
    fn an_archived_change_drops_its_date() {
        assert_eq!(
            named("openspec/changes/archive/2026-10-20-add-thing/proposal.md"),
            Some(Named::Archived { name: "add-thing" })
        );
    }

    #[test]
    fn other_plans_name_no_change() {
        assert_eq!(named("goals/row-edit/plan.md"), None);
        assert_eq!(named("openspec/changes/proposal.md"), None);
        assert_eq!(named("openspec/specs/ledger/spec.md"), None);
        assert_eq!(named("openspec/changes/archive/undated/proposal.md"), None);
    }

    #[test]
    fn an_archive_folder_needs_a_date_and_a_name() {
        assert_eq!(archived_name("2026-10-20-add-thing"), Some("add-thing"));
        assert_eq!(archived_name("2026-10-20-"), None);
        assert_eq!(archived_name("2026-1-20-add-thing"), None);
        assert_eq!(archived_name("add-thing"), None);
    }
}
