//! What a queued row may say, one case per way of saying it wrong.

mod common;

use common::{queued, refused};
use indoc::indoc;
use rstest::rstest;

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
