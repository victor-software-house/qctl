//! What a horizon row may say.

#[path = "common/fixtures.rs"]
mod common;

use common::{on_the_horizon, refused};
use indoc::indoc;
use rstest::rstest;

#[rstest]
#[case::a_reason_it_has_no_word_for(
    indoc! {"
        - id: QCTL-900
          title: Someday
          scope: qctl
          outcome: Something would be true.
          reason: someday
          open: Nobody knows yet.
    "},
    "reason"
)]
#[case::no_reason_at_all(
    indoc! {"
        - id: QCTL-900
          title: Someday
          scope: qctl
          outcome: Something would be true.
          open: Nobody knows yet.
    "},
    "reason"
)]
#[case::nothing_left_open(
    indoc! {r#"
        - id: QCTL-900
          title: Someday
          scope: qctl
          outcome: Something would be true.
          reason: research
          open: ""
    "#},
    "open"
)]
fn refuses_a_horizon_row(#[case] row: &str, #[case] field: &str) {
    refused(&on_the_horizon(row), field);
}
