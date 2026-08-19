//! Shared fixtures. Each integration binary compiles this module on its own,
//! so anything one of them does not reach looks unused here.
#![allow(dead_code)]

use indoc::{formatdoc, indoc};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

pub struct LedgerDir {
    _root: TempDir,
    pub path: PathBuf,
}

impl LedgerDir {
    pub fn empty() -> Self {
        let root = TempDir::new().expect("tempdir");
        let path = root.path().join("tasks.yaml");
        Self { _root: root, path }
    }

    pub fn write(&self, body: &str) {
        fs::write(&self.path, body).expect("write ledger");
    }

    pub fn parent(&self) -> &Path {
        self.path.parent().expect("parent")
    }

    pub fn read(&self) -> String {
        fs::read_to_string(&self.path).expect("read ledger")
    }

    /// Put a document beside the ledger, for the rows that point at one.
    pub fn plant(&self, relative: &str, body: &str) {
        let path = self.parent().join(relative);
        fs::create_dir_all(path.parent().expect("parent")).expect("create dirs");
        fs::write(path, body).expect("write document");
    }
}

pub fn qctl(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_qctl"))
        .args(args)
        .env_remove("TASKS_LEDGER")
        .output()
        .expect("spawn qctl")
}

pub fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

pub const MINIMAL: &str = indoc! {"
    schema_version: 1
    prefix: QCTL
    active: null
    queue: []
    archive: []
    horizon: []
"};

/// A queued row on its own, in the smallest ledger that can hold one.
pub fn queued(row: &str) -> String {
    formatdoc! {"
        schema_version: 1
        prefix: QCTL
        active: null
        queue:
        {row}
        archive: []
    ", row = nested(row)}
}

/// An archived row on its own.
pub fn archived(row: &str) -> String {
    formatdoc! {"
        schema_version: 1
        prefix: QCTL
        active: null
        queue: []
        archive:
        {row}
    ", row = nested(row)}
}

/// A horizon row on its own.
pub fn on_the_horizon(row: &str) -> String {
    formatdoc! {"
        schema_version: 1
        prefix: QCTL
        active: null
        queue: []
        archive: []
        horizon:
        {row}
    ", row = nested(row)}
}

/// A row as written in a case, indented to sit under its section.
fn nested(row: &str) -> String {
    row.trim_end()
        .lines()
        .map(|line| format!("  {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Whether `qctl check` accepts this ledger, and what it said.
pub fn check(body: &str) -> (bool, String) {
    check_in(&LedgerDir::empty(), body)
}

/// The same, in a directory a case has already prepared — a row naming a plan
/// needs that document to be there.
pub fn check_in(dir: &LedgerDir, body: &str) -> (bool, String) {
    dir.write(body);
    let path = dir.path.to_string_lossy().into_owned();
    let output = qctl(&["check", "--file", &path, "--no-git"]);
    (output.status.success(), stderr(&output))
}

pub fn accepted(body: &str) {
    let (ok, complaint) = check(body);
    assert!(ok, "refused:\n{complaint}");
}

/// Refused, and the complaint names the field — "invalid" that does not say
/// where is a bug report, not a message.
pub fn refused(body: &str, field: &str) {
    let (ok, complaint) = check(body);
    assert!(!ok, "accepted, but should not have been:\n{body}");
    assert!(
        complaint.contains(field),
        "the complaint never names {field:?}:\n{complaint}"
    );
}
