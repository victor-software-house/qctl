//! What an archived row may say.

mod common;

use common::{archived, refused};
use indoc::indoc;
use rstest::rstest;

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
