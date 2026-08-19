//! The ledger contract: what it accepts, what it refuses, and that the
//! committed schema is still the one the types describe.
//!
//! Each case is the YAML it is about. A row is written out as a reader would
//! write it, dropped into the smallest ledger that can hold it, and checked;
//! the case says which field the complaint has to name. Nothing here edits a
//! document by hand, so no case can drift away from what it claims to test.

mod common;

use common::{LedgerDir, qctl, stderr};
use indoc::{formatdoc, indoc};
use rstest::rstest;

/// One ledger using every field of every row kind, valid in all of them: the
/// first queued row in block style with every optional field, the second with
/// only what is required and its lists inline.
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
        completed: 2026-08-01
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

/// A queued row on its own, in the smallest ledger that can hold one.
fn queued(row: &str) -> String {
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
fn archived(row: &str) -> String {
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
fn on_the_horizon(row: &str) -> String {
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
fn check(body: &str) -> (bool, String) {
    check_in(&LedgerDir::empty(), body)
}

/// The same, in a directory a case has already prepared — a row naming a plan
/// needs that document to be there.
fn check_in(dir: &LedgerDir, body: &str) -> (bool, String) {
    dir.write(body);
    let path = dir.path.to_string_lossy().into_owned();
    let output = qctl(&["check", "--file", &path, "--no-git"]);
    (output.status.success(), stderr(&output))
}

fn accepted(body: &str) {
    let (ok, complaint) = check(body);
    assert!(ok, "refused:\n{complaint}");
}

/// Refused, and the complaint names the field — "invalid" that does not say
/// where is a bug report, not a message.
fn refused(body: &str, field: &str) {
    let (ok, complaint) = check(body);
    assert!(!ok, "accepted, but should not have been:\n{body}");
    assert!(
        complaint.contains(field),
        "the complaint never names {field:?}:\n{complaint}"
    );
}

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
    schema_version: 1
    prefix: QCTL
    active: null
    queue: []
    archive: []
"})]
#[case::a_ledger_with_no_horizon_section(indoc! {"
    schema_version: 1
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
    schema_version: 1
    prefix: QCTL
    active: null
    queue: []
    archive:
      - id: QCTL-001
        title: Shipped
        scope: qctl
        completed: 2026-08-01
        outcome: It shipped.
        evidence: [The tag exists.]
"})]
fn accepts_a_ledger(#[case] ledger: &str) {
    accepted(ledger);
}

#[rstest]
#[case::an_id_that_is_not_an_id(
    indoc! {"
        - id: QCTL-1
          title: A row
          scope: qctl
          outcome: Something is true.
          blocked_by: []
          acceptance: [It holds.]
    "},
    "id"
)]
#[case::a_row_with_no_id(
    indoc! {"
        - title: A row with no id
          scope: qctl
          outcome: Something is true.
          blocked_by: []
          acceptance: [It holds.]
    "},
    "id"
)]
#[case::an_empty_title(
    indoc! {r#"
        - id: QCTL-001
          title: ""
          scope: qctl
          outcome: Something is true.
          blocked_by: []
          acceptance: [It holds.]
    "#},
    "title"
)]
#[case::an_empty_scope(
    indoc! {r#"
        - id: QCTL-001
          title: A row
          scope: ""
          outcome: Something is true.
          blocked_by: []
          acceptance: [It holds.]
    "#},
    "scope"
)]
#[case::no_outcome_at_all(
    indoc! {"
        - id: QCTL-001
          title: A row
          scope: qctl
          blocked_by: []
          acceptance: [It holds.]
    "},
    "outcome"
)]
#[case::nothing_to_accept(
    indoc! {"
        - id: QCTL-001
          title: A row
          scope: qctl
          outcome: Something is true.
          blocked_by: []
          acceptance: []
    "},
    "acceptance"
)]
#[case::an_empty_acceptance_line(
    indoc! {"
        - id: QCTL-001
          title: A row
          scope: qctl
          outcome: Something is true.
          blocked_by: []
          acceptance: ['']
    "},
    "acceptance"
)]
#[case::the_same_acceptance_line_twice(
    indoc! {"
        - id: QCTL-001
          title: A row
          scope: qctl
          outcome: Something is true.
          blocked_by: []
          acceptance: [It holds., It holds.]
    "},
    "acceptance"
)]
#[case::the_same_blocker_twice(
    indoc! {"
        - id: QCTL-001
          title: A row
          scope: qctl
          outcome: Something is true.
          blocked_by: [QCTL-000, QCTL-000]
          acceptance: [It holds.]
    "},
    "blocked_by"
)]
#[case::a_blocker_that_is_not_an_id(
    indoc! {"
        - id: QCTL-001
          title: A row
          scope: qctl
          outcome: Something is true.
          blocked_by: [the other one]
          acceptance: [It holds.]
    "},
    "blocked_by"
)]
#[case::a_key_the_contract_does_not_have(
    indoc! {"
        - id: QCTL-001
          title: A row
          scope: qctl
          outcome: Something is true.
          blocked_by: []
          acceptance: [It holds.]
          owner: me
    "},
    "owner"
)]
#[case::a_changeset_name_that_is_not_a_file_stem(
    indoc! {"
        - id: QCTL-001
          title: A row
          scope: qctl
          outcome: Something is true.
          blocked_by: []
          acceptance: [It holds.]
          patch: A Changeset Name
    "},
    "patch"
)]
#[case::a_plan_outside_the_repository(
    indoc! {"
        - id: QCTL-001
          title: A row
          scope: qctl
          outcome: Something is true.
          blocked_by: []
          acceptance: [It holds.]
          plan: ../elsewhere/plan.md
    "},
    "plan"
)]
#[case::a_plan_that_is_not_a_document(
    indoc! {"
        - id: QCTL-001
          title: A row
          scope: qctl
          outcome: Something is true.
          blocked_by: []
          acceptance: [It holds.]
          plan: docs/plan.txt
    "},
    "plan"
)]
#[case::an_empty_note(
    indoc! {r#"
        - id: QCTL-001
          title: A row
          scope: qctl
          outcome: Something is true.
          blocked_by: []
          acceptance: [It holds.]
          notes: ""
    "#},
    "notes"
)]
#[case::a_link_that_is_not_a_url(
    indoc! {"
        - id: QCTL-001
          title: A row
          scope: qctl
          outcome: Something is true.
          blocked_by: []
          acceptance: [It holds.]
          links: [see the wiki]
    "},
    "links"
)]
fn refuses_a_queued_row(#[case] row: &str, #[case] field: &str) {
    refused(&queued(row), field);
}

#[rstest]
#[case::a_completed_that_is_not_a_date(
    indoc! {"
        - id: QCTL-001
          title: Shipped
          scope: qctl
          completed: last Tuesday
          outcome: It shipped.
          evidence: [The tag exists.]
    "},
    "completed"
)]
#[case::a_day_that_never_happened(
    indoc! {"
        - id: QCTL-001
          title: Shipped
          scope: qctl
          completed: 2026-02-30
          outcome: It shipped.
          evidence: [The tag exists.]
    "},
    "completed"
)]
#[case::no_completed_date(
    indoc! {"
        - id: QCTL-001
          title: Shipped
          scope: qctl
          outcome: It shipped.
          evidence: [The tag exists.]
    "},
    "completed"
)]
#[case::no_evidence(
    indoc! {"
        - id: QCTL-001
          title: Shipped
          scope: qctl
          completed: 2026-08-01
          outcome: It shipped.
          evidence: []
    "},
    "evidence"
)]
#[case::a_disposition_it_has_no_word_for(
    indoc! {"
        - id: QCTL-001
          title: Shipped
          scope: qctl
          completed: 2026-08-01
          outcome: It shipped.
          evidence: [The tag exists.]
          disposition: abandoned
    "},
    "disposition"
)]
fn refuses_an_archived_row(#[case] row: &str, #[case] field: &str) {
    refused(&archived(row), field);
}

#[rstest]
#[case::a_kind_it_has_no_word_for(
    indoc! {"
        - id: QCTL-900
          title: Someday
          scope: qctl
          outcome: Something would be true.
          kind: someday
          open: Nobody knows yet.
    "},
    "kind"
)]
#[case::no_kind_at_all(
    indoc! {"
        - id: QCTL-900
          title: Someday
          scope: qctl
          outcome: Something would be true.
          open: Nobody knows yet.
    "},
    "kind"
)]
#[case::nothing_left_open(
    indoc! {r#"
        - id: QCTL-900
          title: Someday
          scope: qctl
          outcome: Something would be true.
          kind: research
          open: ""
    "#},
    "open"
)]
fn refuses_a_horizon_row(#[case] row: &str, #[case] field: &str) {
    refused(&on_the_horizon(row), field);
}

#[rstest]
#[case::a_key_the_root_does_not_have(
    indoc! {"
        schema_version: 1
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
#[case::a_schema_version_from_another_era(
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
        schema_version: 1
        prefix: qctl
        active: null
        queue: []
        archive: []
    "},
    "prefix"
)]
#[case::a_prefix_longer_than_the_ids_allow(
    indoc! {"
        schema_version: 1
        prefix: QCTLQCTLQ
        active: null
        queue: []
        archive: []
    "},
    "prefix"
)]
#[case::no_active_line(
    indoc! {"
        schema_version: 1
        prefix: QCTL
        queue: []
        archive: []
    "},
    "active"
)]
#[case::an_active_that_is_not_an_id(
    indoc! {"
        schema_version: 1
        prefix: QCTL
        active: whatever
        queue: []
        archive: []
    "},
    "active"
)]
#[case::no_queue(
    indoc! {"
        schema_version: 1
        prefix: QCTL
        active: null
        archive: []
    "},
    "queue"
)]
#[case::no_archive(
    indoc! {"
        schema_version: 1
        prefix: QCTL
        active: null
        queue: []
    "},
    "archive"
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

/// A ledger the verbs could otherwise act on — the queued row is startable and
/// archivable — held back by one wrong value somewhere else in the file.
const ONE_WRONG_VALUE: &str = indoc! {"
    schema_version: 1
    prefix: QCTL
    active: null
    queue:
      - id: QCTL-002
        title: A row a verb could act on
        scope: qctl
        outcome: Something is true.
        blocked_by: []
        acceptance: [It holds.]
    archive:
      - id: QCTL-001
        title: Shipped
        scope: qctl
        completed: last Tuesday
        outcome: It shipped.
        evidence: [The tag exists.]
"};

/// `check` reads the contract from the schema; the verbs read the same
/// declarations through garde. A verb has to refuse a ledger whose values are
/// wrong and leave the file as it found it — editing one it cannot vouch for is
/// how a bad value becomes a rewritten file. The row it names here is otherwise
/// perfectly actionable, so refusing can only be the validation.
#[rstest]
#[case::start(&["start", "QCTL-002"])]
#[case::add(&["add", "-t", "t", "-s", "s", "-o", "o", "-a", "a"])]
fn a_verb_refuses_a_ledger_it_cannot_vouch_for(#[case] verb: &[&str]) {
    let dir = LedgerDir::empty();
    dir.write(ONE_WRONG_VALUE);

    let path = dir.path.to_string_lossy().into_owned();
    let args: Vec<&str> = verb.iter().copied().chain(["--file", &path]).collect();
    let output = qctl(&args);

    assert!(!output.status.success(), "the verb ran anyway");
    assert_eq!(dir.read(), ONE_WRONG_VALUE, "the verb wrote to it anyway");
}

/// `check` and the verbs read the contract through different engines, and only
/// one of them is what a consumer runs. So every rule garde knows has to be a
/// rule the schema knows too: a value the verbs refuse must never pass `check`,
/// or a defect would be invisible until someone tried to edit the file. These
/// two are the rules garde states in Rust rather than as a keyword, which is
/// where the two could drift apart.
#[rstest]
#[case::a_plan_outside_the_repository(indoc! {"
    schema_version: 1
    prefix: QCTL
    active: null
    queue:
      - id: QCTL-002
        title: A row a verb could act on
        scope: qctl
        outcome: Something is true.
        blocked_by: []
        acceptance: [It holds.]
        plan: ../elsewhere/plan.md
    archive: []
"})]
#[case::a_completed_that_is_not_a_date(ONE_WRONG_VALUE)]
fn check_refuses_whatever_a_verb_refuses(#[case] ledger: &str) {
    let dir = LedgerDir::empty();
    dir.write(ledger);
    let path = dir.path.to_string_lossy().into_owned();

    let verb = qctl(&["start", "QCTL-002", "--file", &path]);
    assert!(!verb.status.success(), "the verb accepted it");

    let (ok, complaint) = check_in(&dir, ledger);
    assert!(
        !ok,
        "the verb refused this and check did not:\n{ledger}\n{complaint}"
    );
}

/// `archive` is the one verb that edits a file it never loaded, so the wrong
/// value above does not stop it. Ignored rather than deleted: the gap is real,
/// it is an acceptance criterion of QCTL-002, and this is the check that will
/// say when it closes.
#[test]
#[ignore = "QCTL-002: archive edits without loading, so nothing vouches for the file"]
fn archive_refuses_a_ledger_it_cannot_vouch_for() {
    let dir = LedgerDir::empty();
    dir.write(ONE_WRONG_VALUE);

    let path = dir.path.to_string_lossy().into_owned();
    let output = qctl(&["archive", "QCTL-002", "-e", "it shipped", "--file", &path]);

    assert!(!output.status.success(), "the verb ran anyway");
    assert_eq!(dir.read(), ONE_WRONG_VALUE, "the verb wrote to it anyway");
}

/// One run has to say everything that is wrong. A reader who fixes the first
/// complaint and runs again should not meet a second one that was known all
/// along — and a check that stops at the first rule cannot be trusted to have
/// looked at the rest.
#[test]
fn reports_every_defect_in_one_pass() {
    let ledger = indoc! {r#"
        schema_version: 1
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
