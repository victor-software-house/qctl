//! The ledger contract: what the types accept, what they refuse, and that the
//! committed schema is still the one they describe.
//!
//! Every rejection case starts from [`WHOLE`] — the same ledger, exercising
//! every field the contract has — and changes exactly one thing. That way a
//! failure names the field that broke rather than a fixture that never parsed.

mod common;

use common::{LedgerDir, qctl, stderr};
use indoc::indoc;

/// One ledger using every field of every row kind, valid in all of them.
const WHOLE: &str = indoc! {r#"
    schema_version: 1
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
        links:
          - https://github.com/victor-software-house/qctl
        notes: Context the other fields cannot carry.
      - id: QCTL-002
        title: Second
        scope: qctl
        outcome: Second outcome.
        blocked_by: [QCTL-001]
        acceptance:
          - It also holds.
    archive:
      - id: QCTL-000
        title: Shipped earlier
        scope: qctl
        completed: 2026-08-01
        outcome: It shipped.
        evidence:
          - The tag exists.
        disposition: completed
    horizon:
      - id: QCTL-900
        title: Someday
        scope: qctl
        outcome: Something would be true.
        kind: research
        open: Nobody knows the answer yet.
"#};

fn check(body: &str) -> (bool, String) {
    let dir = LedgerDir::empty();
    dir.write(body);
    let path = dir.path.to_string_lossy().into_owned();
    let output = qctl(&["check", "--file", &path, "--no-git"]);
    (output.status.success(), stderr(&output))
}

/// Replace the first occurrence of `from` with `to`, failing loudly when the
/// fixture no longer contains it — a rejection test that silently stopped
/// changing anything would pass for the wrong reason.
fn but(from: &str, to: &str) -> String {
    assert!(
        WHOLE.contains(from),
        "the fixture no longer contains {from:?}"
    );
    WHOLE.replacen(from, to, 1)
}

#[test]
fn the_whole_contract_is_accepted() {
    let (ok, complaint) = check(WHOLE);
    assert!(ok, "{complaint}");
}

#[test]
fn a_paused_ledger_is_accepted() {
    let (ok, complaint) = check(&but("active: QCTL-001", "active: null"));
    assert!(ok, "{complaint}");
}

#[test]
fn refuses_a_key_the_contract_does_not_have() {
    let (ok, complaint) = check(&but(
        "    scope: qctl\n",
        "    scope: qctl\n    owner: me\n",
    ));
    assert!(!ok);
    assert!(complaint.contains("owner"), "{complaint}");
}

#[test]
fn refuses_an_id_that_is_not_an_id() {
    let (ok, complaint) = check(&but("id: QCTL-001", "id: QCTL-1"));
    assert!(!ok);
    assert!(complaint.contains("id"), "{complaint}");
}

#[test]
fn refuses_a_lowercase_prefix() {
    let (ok, complaint) = check(&but("prefix: QCTL", "prefix: qctl"));
    assert!(!ok);
    assert!(complaint.contains("prefix"), "{complaint}");
}

#[test]
fn refuses_an_empty_title() {
    let (ok, complaint) = check(&but(
        r#"title: "Quoted: a title with a colon""#,
        r#"title: """#,
    ));
    assert!(!ok);
    assert!(complaint.contains("title"), "{complaint}");
}

#[test]
fn refuses_a_row_with_nothing_to_accept() {
    let (ok, complaint) = check(&but(
        "    acceptance:\n      - It holds.\n",
        "    acceptance: []\n",
    ));
    assert!(!ok);
    assert!(complaint.contains("acceptance"), "{complaint}");
}

#[test]
fn refuses_a_plan_outside_the_repository() {
    let (ok, complaint) = check(&but(
        "    patch: a-changeset-name\n",
        "    plan: ../elsewhere/plan.md\n",
    ));
    assert!(!ok);
    assert!(complaint.contains("plan"), "{complaint}");
}

#[test]
fn refuses_a_plan_that_is_not_a_document() {
    let (ok, complaint) = check(&but(
        "    patch: a-changeset-name\n",
        "    plan: docs/plan.txt\n",
    ));
    assert!(!ok);
    assert!(complaint.contains("plan"), "{complaint}");
}

#[test]
fn refuses_a_changeset_name_that_is_not_a_file_stem() {
    let (ok, complaint) = check(&but("patch: a-changeset-name", "patch: A Changeset Name"));
    assert!(!ok);
    assert!(complaint.contains("patch"), "{complaint}");
}

#[test]
fn refuses_a_completed_date_that_is_not_a_date() {
    let (ok, complaint) = check(&but("completed: 2026-08-01", "completed: last Tuesday"));
    assert!(!ok);
    assert!(complaint.contains("completed"), "{complaint}");
}

#[test]
fn refuses_a_disposition_it_has_no_word_for() {
    let (ok, complaint) = check(&but("disposition: completed", "disposition: abandoned"));
    assert!(!ok);
    assert!(complaint.contains("disposition"), "{complaint}");
}

#[test]
fn refuses_a_horizon_kind_it_has_no_word_for() {
    let (ok, complaint) = check(&but("kind: research", "kind: someday"));
    assert!(!ok);
    assert!(complaint.contains("kind"), "{complaint}");
}

#[test]
fn refuses_a_schema_version_from_another_era() {
    let (ok, complaint) = check(&but("schema_version: 1", "schema_version: 2"));
    assert!(!ok);
    assert!(complaint.contains("schema_version"), "{complaint}");
}

#[test]
fn refuses_the_same_blocker_twice() {
    let (ok, complaint) = check(&but(
        "blocked_by: [QCTL-001]",
        "blocked_by: [QCTL-001, QCTL-001]",
    ));
    assert!(!ok);
    assert!(complaint.contains("blocked_by"), "{complaint}");
}

/// The committed schema is generated, so a change to the types that forgets to
/// regenerate it is a defect in itself: editors would read one contract while
/// the binary enforced another.
#[test]
fn the_committed_schema_is_what_the_types_say() {
    let generated = qctl::schema::generated().expect("generate");
    let committed = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(qctl::schema::COMMITTED),
    )
    .expect("read committed schema");
    assert_eq!(
        committed, generated,
        "run `qctl schema` and commit the result"
    );
}
