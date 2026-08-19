//! What a horizon row may say.

mod common;

use common::{on_the_horizon, refused};
use indoc::indoc;
use rstest::rstest;

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
