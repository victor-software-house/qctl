mod common;

use common::{LedgerDir, qctl, stderr};
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
    assert!(body.contains("schema_version: 4"), "{body}");
    assert_eq!(body.matches("- |-").count(), 2, "{body}");
    assert!(body.contains("First paragraph."), "{body}");
    assert!(body.contains("Second paragraph."), "{body}");
    let check = qctl(&["check", "-f", &path, "-g"]);
    assert!(check.status.success(), "{}", stderr(&check));
}
