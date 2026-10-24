//! Rows and `openspec` changes agree: `check` reports where they do not,
//! `archive` will not leave a change open by accident, and
//! `close-from-openspec` closes and repoints rows after `openspec archive`.

mod common;

use std::process::Command;

use common::{LedgerDir, qctl, stderr, stdout};
use indoc::{formatdoc, indoc};

const OPEN: &str = "openspec/changes/add-thing/proposal.md";
const ARCHIVED: &str = "openspec/changes/archive/2026-10-23-add-thing/proposal.md";

/// A ledger with these queue, archive, and horizon rows, each list written as
/// its YAML items or `[]`.
fn ledger(active: &str, queue: &str, archive: &str, horizon: &str) -> String {
    formatdoc! {"
        schema_version: 5
        prefix: DEMO
        active: {active}
        queue:{queue}
        archive:{archive}
        horizon:{horizon}
    ",
        queue = list(queue),
        archive = list(archive),
        horizon = list(horizon),
    }
}

fn list(items: &str) -> String {
    if items.trim().is_empty() {
        return " []".to_owned();
    }
    let indented: Vec<String> = items
        .trim_end()
        .lines()
        .map(|line| format!("  {line}"))
        .collect();
    format!("\n{}", indented.join("\n"))
}

fn queued(id: &str, plan: &str) -> String {
    formatdoc! {"
        - id: {id}
          title: Add the thing
          scope: demo
          outcome: The thing exists.
          blocked_by: []
          acceptance:
            - The thing exists.
          plan: {plan}
    "}
}

fn archived(id: &str, plan: &str, extra: &str) -> String {
    formatdoc! {"
        - id: {id}
          title: Add the thing
          scope: demo
          completed: 2026-10-20T10:00:00
          outcome: The thing exists.
          evidence:
            - https://example.com/pull/1
          plan: {plan}
        {extra}
    ", extra = if extra.is_empty() { String::new() } else { format!("  {extra}") }}
}

fn on_the_horizon(id: &str, plan: &str) -> String {
    formatdoc! {"
        - id: {id}
          title: Add the thing later
          scope: demo
          outcome: The thing exists.
          kind: deferred
          open: Not now.
          plan: {plan}
    "}
}

/// A ledger directory beside an `openspec/` folder holding `changes`, each a
/// path to a proposal.
fn beside(changes: &[&str]) -> LedgerDir {
    let dir = LedgerDir::empty();
    dir.plant("openspec/config.yaml", "schema: spec-driven\n");
    for change in changes {
        dir.plant(change, "# Proposal\n");
    }
    dir
}

fn run(dir: &LedgerDir, args: &[&str]) -> std::process::Output {
    let path = dir.path.to_string_lossy().into_owned();
    let mut command = args.to_vec();
    command.extend(["--file", &path]);
    qctl(&command)
}

fn check(dir: &LedgerDir) -> (bool, String) {
    let output = run(dir, &["check", "--no-git"]);
    (output.status.success(), stderr(&output))
}

#[test]
fn a_ledger_without_openspec_gets_no_agreement_rule() {
    let dir = LedgerDir::empty();
    dir.plant("goals/plan.md", "# Plan\n");
    dir.write(&ledger(
        "null",
        &queued("DEMO-001", "goals/plan.md"),
        "",
        "",
    ));
    let (ok, complaint) = check(&dir);
    assert!(ok, "{complaint}");
}

#[test]
fn no_flag_turns_the_rules_off() {
    let dir = beside(&[]);
    dir.write(&ledger("null", "", "", ""));
    let output = run(&dir, &["check", "--no-openspec"]);
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("--no-openspec"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn the_last_finished_row_leaves_its_change_open() {
    let dir = beside(&[OPEN]);
    dir.write(&ledger("null", "", &archived("DEMO-001", OPEN, ""), ""));
    let (ok, complaint) = check(&dir);
    assert!(!ok);
    assert!(
        complaint.contains(
            "DEMO-001 is done but its change add-thing is still open; archive the change with openspec archive and run qctl close-from-openspec, or set keep_change_open with a reason"
        ),
        "{complaint}"
    );
}

#[test]
fn a_shared_change_stays_open_while_a_row_still_needs_it() {
    let dir = beside(&[OPEN]);
    dir.write(&ledger(
        "null",
        &queued("DEMO-002", OPEN),
        &archived("DEMO-001", OPEN, ""),
        "",
    ));
    let (ok, complaint) = check(&dir);
    assert!(ok, "{complaint}");
}

#[test]
fn a_parked_row_keeps_its_change_open_too() {
    let dir = beside(&[OPEN]);
    dir.write(&ledger(
        "null",
        "",
        &archived("DEMO-001", OPEN, ""),
        &on_the_horizon("DEMO-002", OPEN),
    ));
    let (ok, complaint) = check(&dir);
    assert!(ok, "{complaint}");
}

#[test]
fn a_reason_keeps_the_change_open() {
    let dir = beside(&[OPEN]);
    dir.write(&ledger(
        "null",
        "",
        &archived(
            "DEMO-001",
            OPEN,
            "keep_change_open: The change continues under another ledger.",
        ),
        "",
    ));
    let (ok, complaint) = check(&dir);
    assert!(ok, "{complaint}");
}

#[test]
fn an_empty_reason_is_refused() {
    let dir = beside(&[OPEN]);
    dir.write(&ledger(
        "null",
        "",
        &archived("DEMO-001", OPEN, "keep_change_open: ''"),
        "",
    ));
    let (ok, complaint) = check(&dir);
    assert!(!ok);
    assert!(complaint.contains("keep_change_open"), "{complaint}");
}

#[test]
fn a_change_no_row_names_is_reported() {
    let dir = beside(&["openspec/changes/toml-input/proposal.md"]);
    dir.write(&ledger("null", "", "", ""));
    let (ok, complaint) = check(&dir);
    assert!(!ok);
    assert!(
        complaint.contains(
            "change toml-input is named by no row; point a row's plan at openspec/changes/toml-input/proposal.md"
        ),
        "{complaint}"
    );
}

#[test]
fn a_plan_into_an_archived_change_names_the_archive() {
    let dir = beside(&[ARCHIVED]);
    dir.write(&ledger("null", &queued("DEMO-001", OPEN), "", ""));
    let (ok, complaint) = check(&dir);
    assert!(!ok);
    assert!(
        complaint.contains(
            "missing plan openspec/changes/add-thing/proposal.md; change add-thing was archived to openspec/changes/archive/2026-10-23-add-thing/; run qctl close-from-openspec"
        ),
        "{complaint}"
    );
}

#[test]
fn a_change_archived_twice_names_both_folders() {
    let dir = beside(&[
        ARCHIVED,
        "openspec/changes/archive/2026-09-01-add-thing/proposal.md",
    ]);
    dir.write(&ledger("null", &queued("DEMO-001", OPEN), "", ""));
    let (ok, complaint) = check(&dir);
    assert!(!ok);
    assert!(
        complaint.contains("change add-thing was archived more than once, to openspec/changes/archive/2026-09-01-add-thing, openspec/changes/archive/2026-10-23-add-thing"),
        "{complaint}"
    );
}

#[test]
fn archive_refuses_the_last_row_of_an_open_change() {
    let dir = beside(&[OPEN]);
    let body = ledger("null", &queued("DEMO-001", OPEN), "", "");
    dir.write(&body);
    let output = run(
        &dir,
        &["archive", "DEMO-001", "-e", "https://example.com/pull/1"],
    );
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains(
            "DEMO-001 is the last row naming open change add-thing; archive the change first (openspec archive add-thing, then qctl close-from-openspec), or pass --keep-change-open REASON"
        ),
        "{}",
        stderr(&output)
    );
    assert_eq!(dir.read(), body);
}

#[test]
fn archive_with_a_reason_keeps_the_change_open() {
    let dir = beside(&[OPEN]);
    dir.write(&ledger("null", &queued("DEMO-001", OPEN), "", ""));
    let output = run(
        &dir,
        &[
            "archive",
            "DEMO-001",
            "-e",
            "https://example.com/pull/1",
            "--keep-change-open",
            "The change continues under another ledger.",
        ],
    );
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(
        dir.read()
            .contains("keep_change_open: The change continues under another ledger."),
        "{}",
        dir.read()
    );
    let (ok, complaint) = check(&dir);
    assert!(ok, "{complaint}");
}

#[test]
fn archive_takes_one_row_of_a_shared_change() {
    let dir = beside(&[OPEN]);
    let rows = format!("{}{}", queued("DEMO-001", OPEN), queued("DEMO-002", OPEN));
    dir.write(&ledger("null", &rows, "", ""));
    let output = run(
        &dir,
        &["archive", "DEMO-001", "-e", "https://example.com/pull/1"],
    );
    assert!(output.status.success(), "{}", stderr(&output));
    let (ok, complaint) = check(&dir);
    assert!(ok, "{complaint}");
}

#[test]
fn close_from_openspec_closes_and_repoints() {
    let dir = beside(&[ARCHIVED]);
    dir.write(&ledger(
        "null",
        &queued("DEMO-002", OPEN),
        &archived("DEMO-001", OPEN, ""),
        "",
    ));
    let output = run(&dir, &["close-from-openspec", "--format", "json"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let report: serde_json::Value =
        serde_json::from_str(&stdout(&output)).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(report["archived"], serde_json::json!(["DEMO-002"]));
    assert_eq!(report["repointed"], serde_json::json!(["DEMO-001"]));
    assert_eq!(report["left"], serde_json::json!([]));

    let body = dir.read();
    assert!(!body.contains(OPEN), "{body}");
    assert!(
        body.contains("- openspec/changes/archive/2026-10-23-add-thing/"),
        "{body}"
    );
    assert_eq!(body.matches(ARCHIVED).count(), 2, "{body}");
    let (ok, complaint) = check(&dir);
    assert!(ok, "{complaint}");
}

#[test]
fn close_from_openspec_with_nothing_to_close_writes_nothing() {
    let dir = beside(&[OPEN]);
    let body = ledger("null", &queued("DEMO-001", OPEN), "", "");
    dir.write(&body);
    let output = run(&dir, &["close-from-openspec", "--format", "json"]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(dir.read(), body);
}

#[test]
fn close_from_openspec_leaves_a_parked_row() {
    let dir = beside(&[ARCHIVED]);
    let body = ledger("null", "", "", &on_the_horizon("DEMO-001", OPEN));
    dir.write(&body);
    let output = run(&dir, &["close-from-openspec", "--format", "json"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let report: serde_json::Value =
        serde_json::from_str(&stdout(&output)).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(report["left"], serde_json::json!(["DEMO-001"]));
    assert_eq!(dir.read(), body);
}

#[test]
fn close_from_openspec_refuses_to_close_the_active_row() {
    let dir = beside(&[ARCHIVED]);
    let body = ledger("DEMO-001", &queued("DEMO-001", OPEN), "", "");
    dir.write(&body);
    let output = run(&dir, &["close-from-openspec"]);
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains(
            "active DEMO-001 names archived change add-thing; set active to another row first"
        ),
        "{}",
        stderr(&output)
    );
    assert_eq!(dir.read(), body);
}

#[test]
fn fmt_upgrades_schema_4_by_one_line() {
    let dir = LedgerDir::empty();
    let body = indoc! {"
        schema_version: 4
        prefix: DEMO
        active: null
        queue: []
        archive: []
        horizon: []
    "};
    dir.write(body);
    let status = run(&dir, &["status"]);
    assert!(!status.status.success());
    assert!(
        stderr(&status).contains("schema_version 4; run qctl fmt to set schema_version 5"),
        "{}",
        stderr(&status)
    );
    let output = run(&dir, &["fmt"]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        dir.read(),
        body.replace("schema_version: 4", "schema_version: 5")
    );
}

#[expect(
    clippy::expect_used,
    reason = "a helper that cannot run git fails its test"
)]
fn git(root: &std::path::Path, args: &[&str]) {
    let status = Command::new("git")
        .args(["-C", &root.display().to_string()])
        .args(args)
        .status()
        .expect("git");
    assert!(status.success(), "git {args:?}");
}

#[test]
fn a_trailer_cannot_close_the_last_row_of_an_open_change() {
    let dir = beside(&[OPEN]);
    dir.write(&ledger("null", &queued("DEMO-001", OPEN), "", ""));
    let root = dir.parent();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.email", "t@example.com"]);
    git(root, &["config", "user.name", "t"]);
    git(root, &["add", "-A"]);
    git(
        root,
        &[
            "commit",
            "-q",
            "-m",
            "feat: the thing",
            "-m",
            "Closes DEMO-001",
        ],
    );

    let output = common::qctl_in(root, &["close-from-git", "--format", "json"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let report: serde_json::Value =
        serde_json::from_str(&stdout(&output)).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(report["archived"], serde_json::json!([]));
    assert_eq!(report["refused"], serde_json::json!(["DEMO-001"]));

    let check = common::qctl_in(root, &["check"]);
    assert!(!check.status.success());
    assert!(
        stderr(&check).contains("but its change add-thing is still open; archive the change with openspec archive and run qctl close-from-openspec, or run qctl archive DEMO-001 --keep-change-open REASON"),
        "{}",
        stderr(&check)
    );
}
