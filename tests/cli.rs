//! Binary contract: init, check, add, start, archive.

mod common;

use common::{LedgerDir, MINIMAL, qctl, stderr, stdout};
use indoc::indoc;

#[test]
fn init_writes_ledger_without_schema_copy() {
    let dir = LedgerDir::empty();
    let output = qctl(&["init", "-p", "omx", "-f", dir.path.to_str().unwrap()]);
    assert!(output.status.success(), "{}", stderr(&output));
    let body = dir.read();
    assert!(body.contains("prefix: OMX"));
    assert!(body.contains("horizon: []"));
    assert!(body.contains("$schema="));
    assert!(!dir.parent().join("tasks.schema.json").exists());
}

#[test]
fn init_refuses_overwrite_without_force() {
    let dir = LedgerDir::empty();
    dir.write(MINIMAL);
    let output = qctl(&["init", "-p", "QCTL", "-f", dir.path.to_str().unwrap()]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("already exists"));
}

#[test]
fn check_accepts_own_repo_shape() {
    let dir = LedgerDir::empty();
    dir.write(MINIMAL);
    let output = qctl(&["check", "-f", dir.path.to_str().unwrap()]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains("ok"));
}

#[test]
fn check_accepts_archive_notes() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {"
        schema_version: 1
        prefix: QCTL
        active: null
        queue: []
        archive:
          - id: QCTL-001
            title: done
            scope: s
            completed: '2026-08-17'
            outcome: o
            evidence: [landed]
            notes: >-
              Keep the context that would not fit in evidence.
    "});
    let output = qctl(&["check", "-f", dir.path.to_str().unwrap()]);
    assert!(output.status.success(), "{}", stderr(&output));
}

#[test]
fn check_rejects_unknown_field() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {"
        schema_version: 1
        prefix: QCTL
        active: null
        queue: []
        archive: []
        nope: true
    "});
    let output = qctl(&["check", "-f", dir.path.to_str().unwrap()]);
    assert!(!output.status.success());
}

#[test]
fn add_start_archive_round_trip() {
    let dir = LedgerDir::empty();
    assert!(
        qctl(&["init", "-p", "QCTL", "-f", dir.path.to_str().unwrap()])
            .status
            .success()
    );

    let add = qctl(&[
        "add",
        "-f",
        dir.path.to_str().unwrap(),
        "-t",
        "First",
        "-s",
        "qctl",
        "-o",
        "done",
        "-a",
        "shipped",
    ]);
    assert!(add.status.success(), "{}", stderr(&add));
    assert_eq!(stdout(&add).trim(), "QCTL-001");

    let start = qctl(&["start", "QCTL-001", "-f", dir.path.to_str().unwrap()]);
    assert!(start.status.success(), "{}", stderr(&start));

    let check = qctl(&["check", "-f", dir.path.to_str().unwrap()]);
    assert!(check.status.success(), "{}", stderr(&check));

    let archive = qctl(&[
        "archive",
        "QCTL-001",
        "-f",
        dir.path.to_str().unwrap(),
        "-e",
        "landed",
    ]);
    assert!(archive.status.success(), "{}", stderr(&archive));

    let after = qctl(&["check", "-f", dir.path.to_str().unwrap()]);
    assert!(after.status.success(), "{}", stderr(&after));
    let body = dir.read();
    assert!(body.contains("disposition: completed"));
    assert!(body.contains("QCTL-001"));
}

#[test]
fn start_refuses_blocked_or_horizon() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {"
        schema_version: 1
        prefix: QCTL
        active: null
        queue:
          - id: QCTL-002
            title: blocked
            scope: s
            outcome: o
            blocked_by: [QCTL-001]
            acceptance: [a]
          - id: QCTL-001
            title: first
            scope: s
            outcome: o
            blocked_by: []
            acceptance: [a]
        archive: []
        horizon:
          - id: QCTL-009
            title: later
            scope: s
            outcome: o
            kind: research
            open: unknown
    "});
    let blocked = qctl(&["start", "QCTL-002", "-f", dir.path.to_str().unwrap()]);
    assert!(!blocked.status.success());
    assert!(stderr(&blocked).contains("blocked"));

    let horizon = qctl(&["start", "QCTL-009", "-f", dir.path.to_str().unwrap()]);
    assert!(!horizon.status.success());
    assert!(stderr(&horizon).contains("not queued"));
}

#[test]
fn instructions_prints_contract() {
    let output = qctl(&["instructions"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("horizon"));
    assert!(text.contains("tasks.yaml"));
}

#[test]
fn status_lists_horizon() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {"
        schema_version: 1
        prefix: QCTL
        active: QCTL-001
        queue:
          - id: QCTL-001
            title: now
            scope: s
            outcome: o
            blocked_by: []
            acceptance: [a]
        archive: []
        horizon:
          - id: QCTL-008
            title: later
            scope: s
            outcome: o
            kind: evaluation
            open: wait
    "});
    let output = qctl(&["status", "-f", dir.path.to_str().unwrap()]);
    assert!(output.status.success(), "{}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("QCTL-001"));
    assert!(text.contains("QCTL-008"));
    assert!(text.contains("evaluation"));
}
