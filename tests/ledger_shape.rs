//! The shape of the ledger itself: its root keys, and the ledgers that are
//! whole. Row-level rules live in the rows_* suites.

mod common;

use common::{LedgerDir, accepted, check, check_in, refused};
use indoc::indoc;
use rstest::rstest;

/// One ledger using every field of every row kind, valid in all of them: the
/// first queued row in block style with every optional field, the second with
/// only what is required and its lists inline.
const WHOLE: &str = indoc! {r#"
    schema_version: 3
    prefix: QCTL
    active: QCTL-001
    queue:
      - id: QCTL-001
        title: "Quoted: a title with a colon"
        scope: qctl
        outcome: >-
          A folded outcome that wraps
          across two lines.
        blocked_by: []
        acceptance:
          - It holds.
        patch: a-changeset-name
        plan: docs/plan.md
        links:
          - https://github.com/victor-software-house/qctl
        notes: Context the other fields cannot carry.
      - id: QCTL-002
        title: Second
        scope: release
        outcome: Second outcome.
        blocked_by: [QCTL-001]
        acceptance: [It also holds.]
    archive:
      - id: QCTL-000
        title: Shipped earlier
        scope: qctl
        completed: 2026-08-01T09:12:00
        outcome: It shipped.
        evidence: [The tag exists.]
        disposition: completed
    horizon:
      - id: QCTL-900
        title: Someday
        scope: qctl
        outcome: Something would be true.
        kind: research
        open: Nobody knows the answer yet.
"#};

#[test]
fn every_field_of_every_row_kind_is_accepted() {
    let dir = LedgerDir::empty();
    dir.plant("docs/plan.md", "# The plan\n");
    let (ok, complaint) = check_in(&dir, WHOLE);
    assert!(ok, "refused:\n{complaint}");
}

/// A plan is a document in this repository, so naming one that is not there is
/// a defect the schema cannot see — only the filesystem knows.
#[test]
fn refuses_a_plan_that_was_never_written() {
    let (ok, complaint) = check(WHOLE);
    assert!(!ok, "accepted a plan that does not exist");
    assert!(
        complaint.contains("missing plan docs/plan.md"),
        "{complaint}"
    );
}

#[rstest]
#[case::a_paused_ledger(indoc! {"
    schema_version: 3
    prefix: QCTL
    active: null
    queue: []
    archive: []
"})]
#[case::a_ledger_with_no_horizon_section(indoc! {"
    schema_version: 3
    prefix: QCTL
    active: QCTL-001
    queue:
      - id: QCTL-001
        title: The only row
        scope: qctl
        outcome: Something is true.
        blocked_by: []
        acceptance: [It holds.]
    archive: []
"})]
#[case::an_archived_row_without_a_disposition(indoc! {"
    schema_version: 3
    prefix: QCTL
    active: null
    queue: []
    archive:
      - id: QCTL-001
        title: Shipped
        scope: qctl
        completed: 2026-08-01T09:12:00
        outcome: It shipped.
        evidence: [The tag exists.]
"})]
fn accepts_a_ledger(#[case] ledger: &str) {
    accepted(ledger);
}

#[rstest]
#[case::a_key_the_root_does_not_have(
    indoc! {"
        schema_version: 3
        prefix: QCTL
        active: null
        owner: me
        queue: []
        archive: []
    "},
    "owner"
)]
#[case::no_schema_version(
    indoc! {"
        prefix: QCTL
        active: null
        queue: []
        archive: []
    "},
    "schema_version"
)]
#[case::the_schema_version_this_one_replaced(
    indoc! {"
        schema_version: 2
        prefix: QCTL
        active: null
        queue: []
        archive: []
    "},
    "schema_version"
)]
#[case::a_lowercase_prefix(
    indoc! {"
        schema_version: 3
        prefix: qctl
        active: null
        queue: []
        archive: []
    "},
    "prefix"
)]
#[case::a_prefix_longer_than_the_ids_allow(
    indoc! {"
        schema_version: 3
        prefix: QCTLQCTLQ
        active: null
        queue: []
        archive: []
    "},
    "prefix"
)]
#[case::no_active_line(
    indoc! {"
        schema_version: 3
        prefix: QCTL
        queue: []
        archive: []
    "},
    "active"
)]
#[case::an_active_that_is_not_an_id(
    indoc! {"
        schema_version: 3
        prefix: QCTL
        active: whatever
        queue: []
        archive: []
    "},
    "active"
)]
#[case::no_queue(
    indoc! {"
        schema_version: 3
        prefix: QCTL
        active: null
        archive: []
    "},
    "queue"
)]
#[case::no_archive(
    indoc! {"
        schema_version: 3
        prefix: QCTL
        active: null
        queue: []
    "},
    "archive"
)]
#[case::a_plan_with_a_backslash_parent(
    indoc! {r#"
        schema_version: 3
        prefix: QCTL
        active: null
        queue:
          - id: QCTL-001
            title: t
            scope: qctl
            outcome: o
            blocked_by: []
            acceptance: [a]
            plan: "..\\secret.md"
        archive: []
    "#},
    "does not match"
)]
#[case::a_file_that_does_not_parse(
    indoc! {"
        queue:
          - id: QCTL-001
           title: crooked
    "},
    "parse"
)]
fn refuses_a_ledger(#[case] ledger: &str, #[case] field: &str) {
    refused(ledger, field);
}

/// One run has to say everything that is wrong. A reader who fixes the first
/// complaint and runs again should not meet a second one that was known all
/// along — and a check that stops at the first rule cannot be trusted to have
/// looked at the rest.
#[test]
fn reports_every_defect_in_one_pass() {
    let ledger = indoc! {r#"
        schema_version: 3
        prefix: QCTL
        active: null
        queue:
          - id: QCTL-001
            title: ""
            scope: qctl
            outcome: Something is true.
            blocked_by: [QCTL-000, QCTL-000]
            acceptance: [It holds.]
          - id: QCTL-001
            title: The same id again
            scope: qctl
            outcome: Something else is true.
            blocked_by: []
            acceptance: [It also holds.]
        archive: []
    "#};
    let (ok, complaint) = check(ledger);
    assert!(!ok);
    for expected in ["title", "blocked_by", "duplicate id QCTL-001"] {
        assert!(
            complaint.contains(expected),
            "no {expected} in:\n{complaint}"
        );
    }
}
