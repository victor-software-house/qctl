mod common;

use common::{qctl, qctl_in, stderr};
use indoc::indoc;
use qctl::cli::CheckArgs;
use qctl::trailers;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn git(root: &std::path::Path, args: &[&str]) {
    let status = Command::new("git")
        .args(["-C", &root.display().to_string()])
        .args(args)
        .status()
        .expect("git");
    assert!(status.success(), "git {args:?}");
}

#[test]
fn parse_log_needs_body_trailer() {
    assert_eq!(
        trailers::parse_log(indoc! {"
            abc\0Closes CTC-001 only in subject
        "}),
        Vec::new()
    );
}

#[test]
fn check_fails_when_trailer_closes_queued_id() {
    let root = TempDir::new().expect("tmp");
    git(root.path(), &["init"]);
    git(root.path(), &["config", "user.email", "t@example.com"]);
    git(root.path(), &["config", "user.name", "t"]);
    fs::write(
        root.path().join("tasks.yaml"),
        indoc! {"
            schema_version: 3
            prefix: CTC
            active: CTC-001
            queue:
              - id: CTC-001
                title: Chassis
                scope: ctl-core
                outcome: Crate exists.
                blocked_by: []
                acceptance:
                  - prelude ships
            archive: []
        "},
    )
    .expect("ledger");
    fs::write(root.path().join("note"), "x").expect("note");
    git(root.path(), &["add", "tasks.yaml", "note"]);
    git(
        root.path(),
        &["commit", "-m", "feat: chassis", "-m", "Closes CTC-001"],
    );
    let path = root.path().join("tasks.yaml");
    let output = qctl_in(root.path(), &["check", "-f", path.to_str().expect("utf-8")]);
    assert!(!output.status.success());
    let err = stderr(&output);
    assert!(err.contains("CTC-001"), "{err}");
    assert!(err.contains("still queued"), "{err}");
}

#[test]
fn check_ok_with_no_git() {
    let root = TempDir::new().expect("tmp");
    fs::write(
        root.path().join("tasks.yaml"),
        indoc! {"
            schema_version: 3
            prefix: CTC
            active: null
            queue: []
            archive: []
        "},
    )
    .expect("ledger");
    let args = CheckArgs {
        ledger: qctl::cli::LedgerArgs {
            file: Some(root.path().join("tasks.yaml")),
        },
        no_git: true,
    };
    qctl::check::run(&args).expect("ok");
}

#[test]
fn check_reports_when_the_git_scan_fails() {
    let root = TempDir::new().expect("tmp");
    git(root.path(), &["init"]);
    fs::write(
        root.path().join("tasks.yaml"),
        indoc! {"
            schema_version: 3
            prefix: CTC
            active: null
            queue: []
            archive: []
        "},
    )
    .expect("ledger");
    let path = root.path().join("tasks.yaml");
    let output = qctl_in(root.path(), &["check", "-f", path.to_str().expect("utf-8")]);
    assert!(!output.status.success(), "empty git history must not pass");
    let err = stderr(&output);
    assert!(err.contains("git trailer scan failed"), "{err}");
}

#[test]
fn check_reports_when_the_ledger_is_outside_the_cwd_repo() {
    let ledger_repo = TempDir::new().expect("tmp");
    git(ledger_repo.path(), &["init"]);
    git(
        ledger_repo.path(),
        &["config", "user.email", "t@example.com"],
    );
    git(ledger_repo.path(), &["config", "user.name", "t"]);
    fs::write(
        ledger_repo.path().join("tasks.yaml"),
        indoc! {"
            schema_version: 3
            prefix: CTC
            active: null
            queue: []
            archive: []
        "},
    )
    .expect("ledger");
    git(ledger_repo.path(), &["add", "tasks.yaml"]);
    git(ledger_repo.path(), &["commit", "-m", "init"]);

    let cwd_repo = TempDir::new().expect("cwd");
    git(cwd_repo.path(), &["init"]);
    let path = ledger_repo.path().join("tasks.yaml");
    let output = qctl_in(
        cwd_repo.path(),
        &["check", "-f", path.to_str().expect("utf-8")],
    );
    assert!(!output.status.success(), "foreign ledger must not pass");
    let err = stderr(&output);
    assert!(err.contains("git trailer scan skipped"), "{err}");
    assert!(err.contains("--no-git"), "{err}");
}

#[test]
fn check_without_no_git_does_not_pass_a_scratch_ledger() {
    let dir = TempDir::new().expect("tmp");
    fs::write(
        dir.path().join("tasks.yaml"),
        indoc! {"
            schema_version: 3
            prefix: CTC
            active: null
            queue: []
            archive: []
        "},
    )
    .expect("ledger");
    let path = dir.path().join("tasks.yaml");
    let output = qctl(&["check", "-f", path.to_str().expect("utf-8")]);
    assert!(!output.status.success(), "silence is the bug");
    let err = stderr(&output);
    assert!(err.contains("git trailer scan skipped"), "{err}");
}
