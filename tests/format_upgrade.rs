//! Older ledgers: `fmt` upgrades them, and every other verb refuses them.

#[path = "common/fixtures.rs"]
mod common;

use common::{LedgerDir, qctl, stderr, stdout};
use indoc::indoc;

#[test]
fn other_verbs_refuse_schema_3_and_name_fmt() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {"
        schema_version: 3
        prefix: QCTL
        active: null
        queue: []
        archive: []
        horizon: []
    "});
    let path = dir.path.to_string_lossy();
    let add = qctl(&[
        "add", "-f", &path, "-t", "t", "-s", "s", "-o", "o", "-a", "a",
    ]);
    assert!(!add.status.success());
    assert!(stderr(&add).contains("qctl fmt"), "{}", stderr(&add));
    let check = qctl(&["check", "-f", &path, "-g"]);
    assert!(!check.status.success());
    assert!(stderr(&check).contains("qctl fmt"), "{}", stderr(&check));
}

#[test]
fn fmt_rewrites_v3_scalar_notes_into_a_list() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {"
        schema_version: 3
        prefix: QCTL
        active: null
        queue:
          - id: QCTL-001
            title: t
            scope: s
            outcome: o
            blocked_by: []
            acceptance: [It holds.]
            notes: |-
              First paragraph.

              Second paragraph.
        archive: []
        horizon: []
    "});
    let path = dir.path.to_string_lossy();
    let output = qctl(&["fmt", "-f", &path]);
    assert!(output.status.success(), "{}", stderr(&output));
    let body = dir.read();
    assert!(body.contains("schema_version: 6"), "{body}");
    assert!(body.contains("- First paragraph."), "{body}");
    assert!(body.contains("- Second paragraph."), "{body}");
    let check = qctl(&["check", "-f", &path, "-g"]);
    assert!(check.status.success(), "{}", stderr(&check));
}

/// A schema 5 ledger whose horizon rows still say `kind`, already in its
/// declared style, so the upgrade is the only thing `fmt` changes.
const SCHEMA_5: &str = indoc! {"
    schema_version: 5
    prefix: QCTL
    active: null
    queue: []
    archive: []
    horizon:
      - id: QCTL-001
        title: Choose storage
        scope: design
        outcome: One storage decision is recorded.
        # Facts first.
        kind: research # until the benchmark runs
        open: The benchmark result is missing.

      - id: QCTL-002
        title: Pick a host
        scope: ops
        outcome: One host is chosen.
        kind: deferred
        open: The budget is not approved.
"};

#[test]
fn other_verbs_refuse_schema_5_and_name_fmt() {
    let dir = LedgerDir::empty();
    dir.write(SCHEMA_5);
    let path = dir.path.to_string_lossy();
    let message = "schema_version 5; run qctl fmt to rename the horizon key kind to reason and set schema_version 6";
    let status = qctl(&["status", "-f", &path]);
    assert!(!status.status.success());
    assert!(stderr(&status).contains(message), "{}", stderr(&status));
    let check = qctl(&["check", "-f", &path, "-g", "--format", "json"]);
    assert!(!check.status.success());
    let report: serde_json::Value =
        serde_json::from_str(&stdout(&check)).unwrap_or_else(|error| panic!("{error}"));
    let problems = report["problems"].as_array().cloned().unwrap_or_default();
    assert_eq!(problems.len(), 1, "{report}");
    assert!(
        problems[0]
            .as_str()
            .is_some_and(|problem| problem.ends_with(message)),
        "{report}"
    );
    assert_eq!(dir.read(), SCHEMA_5);
}

#[test]
fn fmt_renames_the_horizon_kind_of_schema_5_in_place() {
    let dir = LedgerDir::empty();
    dir.write(SCHEMA_5);
    let path = dir.path.to_string_lossy();
    let output = qctl(&["fmt", "-f", &path]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        dir.read(),
        SCHEMA_5
            .replace("schema_version: 5", "schema_version: 6")
            .replace("kind:", "reason:")
    );
    let check = qctl(&["check", "-f", &path, "-g"]);
    assert!(check.status.success(), "{}", stderr(&check));
}
