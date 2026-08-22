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
    let args = CheckArgs {
        ledger: qctl::cli::LedgerArgs {
            file: Some(root.path().join("tasks.yaml")),
        },
        no_git: false,
    };
    let error = qctl::check::run(&args).expect_err("stale");
    let message = format!("{error:#}");
    assert!(message.contains("CTC-001"), "{message}");
    assert!(message.contains("still queued"), "{message}");
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
