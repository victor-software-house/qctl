//! Shared fixtures. Each integration binary compiles this module on its own,
//! so anything one of them does not reach looks unused here.
#![allow(
    dead_code,
    reason = "each integration binary reaches only part of this module"
)]
#![expect(
    clippy::expect_used,
    reason = "a fixture that cannot be set up fails its test"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use indoc::{formatdoc, indoc};
use tempfile::TempDir;

pub(crate) struct LedgerDir {
    _root: TempDir,
    pub path: PathBuf,
}

impl LedgerDir {
    pub(crate) fn empty() -> Self {
        let root = TempDir::new().expect("tempdir");
        let path = root.path().join("tasks.yaml");
        Self { _root: root, path }
    }

    pub(crate) fn write(&self, body: &str) {
        fs::write(&self.path, body).expect("write ledger");
    }

    pub(crate) fn parent(&self) -> &Path {
        self.path.parent().expect("parent")
    }

    pub(crate) fn read(&self) -> String {
        fs::read_to_string(&self.path).expect("read ledger")
    }

    /// Put a document beside the ledger, for the rows that point at one.
    pub(crate) fn plant(&self, relative: &str, body: &str) {
        let path = self.parent().join(relative);
        fs::create_dir_all(path.parent().expect("parent")).expect("create dirs");
        fs::write(path, body).expect("write document");
    }
}

/// The variables a caller's shell may set that change what qctl prints. Agent
/// shells export `CLICOLOR_FORCE=1` and `FORCE_COLOR=3`, which colour piped
/// output and break every content check.
const CALLER_ENV: [&str; 5] = [
    "CLICOLOR",
    "CLICOLOR_FORCE",
    "FORCE_COLOR",
    "NO_COLOR",
    "COLORTERM",
];

/// The qctl binary with none of [`CALLER_ENV`] and no `QCTL_*` variable,
/// which every long option reads, so a test sees the same output in an agent
/// shell, a plain terminal, and CI. Every test that starts qctl goes through
/// it; a test about colour or a variable sets its own back on top.
pub(crate) fn binary() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_qctl"));
    for name in CALLER_ENV {
        command.env_remove(name);
    }
    for (name, _) in std::env::vars_os() {
        if name.to_string_lossy().starts_with("QCTL_") {
            command.env_remove(name);
        }
    }
    command
}

/// Content checks run wide, so a message is matched whole rather than across
/// the fallback width's line breaks.
pub(crate) fn qctl(args: &[&str]) -> Output {
    binary()
        .args(args)
        .env("COLUMNS", "1000")
        .output()
        .expect("spawn qctl")
}

pub(crate) fn qctl_in(dir: &Path, args: &[&str]) -> Output {
    binary()
        .current_dir(dir)
        .args(args)
        .env("COLUMNS", "1000")
        .output()
        .expect("spawn qctl")
}

pub(crate) fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub(crate) fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

pub(crate) const MINIMAL: &str = indoc! {"
    schema_version: 4
    prefix: QCTL
    active: null
    queue: []
    archive: []
    horizon: []
"};

/// A queued row on its own, in the smallest ledger that can hold one.
pub(crate) fn queued(row: &str) -> String {
    formatdoc! {"
        schema_version: 4
        prefix: QCTL
        active: null
        queue:
        {row}
        archive: []
    ", row = nested(row)}
}

/// An archived row on its own.
pub(crate) fn archived(row: &str) -> String {
    formatdoc! {"
        schema_version: 4
        prefix: QCTL
        active: null
        queue: []
        archive:
        {row}
    ", row = nested(row)}
}

/// A horizon row on its own.
pub(crate) fn on_the_horizon(row: &str) -> String {
    formatdoc! {"
        schema_version: 4
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
pub(crate) fn check(body: &str) -> (bool, String) {
    check_in(&LedgerDir::empty(), body)
}

/// The same, in a directory a case has already prepared — a row naming a plan
/// needs that document to be there.
pub(crate) fn check_in(dir: &LedgerDir, body: &str) -> (bool, String) {
    dir.write(body);
    let path = dir.path.to_string_lossy().into_owned();
    let output = qctl(&["check", "--file", &path, "--no-git"]);
    (output.status.success(), stderr(&output))
}

pub(crate) fn accepted(body: &str) {
    let (ok, complaint) = check(body);
    assert!(ok, "refused:\n{complaint}");
}

/// Refused, and the complaint names the field — "invalid" that does not say
/// where is a bug report, not a message.
pub(crate) fn refused(body: &str, field: &str) {
    let (ok, complaint) = check(body);
    assert!(!ok, "accepted, but should not have been:\n{body}");
    assert!(
        complaint.contains(field),
        "the complaint never names {field:?}:\n{complaint}"
    );
}
