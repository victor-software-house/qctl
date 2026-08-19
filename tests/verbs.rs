//! The verbs at their boundary: what they refuse to edit, and whether the two
//! engines that read the contract still agree.

mod common;

use common::{LedgerDir, check_in, qctl};
use indoc::indoc;
use rstest::rstest;

const ONE_WRONG_VALUE: &str = indoc! {"
    schema_version: 2
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
    schema_version: 2
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
#[case::a_completed_that_is_not_a_moment(ONE_WRONG_VALUE)]
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

/// `archive` used to be the one verb that edited a file it never loaded, so the
/// wrong value above did not stop it. It loads now, like the other two.
#[test]
fn archive_refuses_a_ledger_it_cannot_vouch_for() {
    let dir = LedgerDir::empty();
    dir.write(ONE_WRONG_VALUE);

    let path = dir.path.to_string_lossy().into_owned();
    let output = qctl(&["archive", "QCTL-002", "-e", "it shipped", "--file", &path]);

    assert!(!output.status.success(), "the verb ran anyway");
    assert_eq!(dir.read(), ONE_WRONG_VALUE, "the verb wrote to it anyway");
}
