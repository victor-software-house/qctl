mod common;

use common::{LedgerDir, qctl, stderr};
use indoc::indoc;

#[test]
fn queued_edit_updates_only_requested_fields_and_preserves_flow_lists() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {"
        schema_version: 4
        prefix: QCTL
        active: QCTL-001
        queue:
          - id: QCTL-001
            title: Old
            scope: core
            outcome: Keep the outcome.
            blocked_by: []
            acceptance: [It holds.]
            notes:
              - First note.
              - Second note.
            links: [https://old.example/a:b]

          - id: QCTL-002
            title: Untouched
            scope: other
            outcome: This row stays byte-identical.
            blocked_by: []
            acceptance: [It holds.]
        archive: []
        horizon: []
    "});
    let untouched_before = dir
        .read()
        .split_once("  - id: QCTL-002")
        .expect("second row")
        .1
        .to_owned();
    let path = dir.path.to_string_lossy();
    let output = qctl(&[
        "edit",
        "QCTL-001",
        "-f",
        &path,
        "-t",
        "New",
        "-n",
        "Third note.",
        "-a",
        "A second condition.",
        "-L",
        "https://new.example",
        "-x",
        "note:1",
        "-x",
        "link:https://old.example/a:b",
        "-R",
        "1",
    ]);
    assert!(output.status.success(), "{}", stderr(&output));

    let body = dir.read();
    assert!(body.contains("title: New"), "{body}");
    assert!(
        body.contains("acceptance: [It holds., A second condition.]"),
        "{body}"
    );
    assert!(!body.contains("First note."), "{body}");
    assert!(body.contains("- Second note."), "{body}");
    assert!(body.contains("- Third note."), "{body}");
    assert!(!body.contains("https://old.example/a:b"), "{body}");
    assert!(body.contains("links: [https://new.example]"), "{body}");
    let untouched_after = body.split_once("  - id: QCTL-002").expect("second row").1;
    assert_eq!(untouched_after, untouched_before);
}

#[test]
fn queue_position_revalidates_every_dependency() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {"
        schema_version: 4
        prefix: QCTL
        active: QCTL-001
        queue:
          - id: QCTL-001
            title: First
            scope: core
            outcome: First.
            blocked_by: []
            acceptance: [It holds.]
          - id: QCTL-002
            title: Second
            scope: core
            outcome: Second.
            blocked_by: [QCTL-001]
            acceptance: [It holds.]
          - id: QCTL-003
            title: Third
            scope: core
            outcome: Third.
            blocked_by: []
            acceptance: [It holds.]
        archive: []
        horizon: []
    "});
    let path = dir.path.to_string_lossy();

    let moved = qctl(&[
        "edit",
        "QCTL-003",
        "-f",
        &path,
        "-b",
        "QCTL-002",
        "-p",
        "after:QCTL-002",
    ]);
    assert!(moved.status.success(), "{}", stderr(&moved));
    let body = dir.read();
    assert!(
        body.find("id: QCTL-002") < body.find("id: QCTL-003"),
        "{body}"
    );

    let refused = qctl(&["edit", "QCTL-002", "-f", &path, "-p", "after:QCTL-003"]);
    assert!(!refused.status.success());
    assert!(stderr(&refused).contains("QCTL-003 <- QCTL-002 is not earlier"));
    assert_eq!(dir.read(), body);
}

#[test]
fn row_kind_policy_is_explicit() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {"
        schema_version: 4
        prefix: QCTL
        active: null
        queue: []
        archive:
          - id: QCTL-001
            title: Done
            scope: core
            completed: 2026-08-01T09:12:00
            outcome: Done.
            evidence: [landed]
            notes: [Keep]
            links: [https://example.com]
        horizon:
          - id: QCTL-002
            title: Later
            scope: core
            kind: research
            outcome: Later.
            open: Need a fact.
            notes: [Old]
    "});
    let path = dir.path.to_string_lossy();

    let archive = qctl(&[
        "edit",
        "QCTL-001",
        "-f",
        &path,
        "-n",
        "New",
        "-e",
        "verified",
        "-x",
        "note:Keep",
        "-x",
        "link:1",
    ]);
    assert!(archive.status.success(), "{}", stderr(&archive));

    let archive_title = qctl(&["edit", "QCTL-001", "-f", &path, "-t", "Nope"]);
    assert!(!archive_title.status.success());
    assert!(stderr(&archive_title).contains("note, evidence, and link"));

    let horizon_position = qctl(&["edit", "QCTL-002", "-f", &path, "-p", "front"]);
    assert!(!horizon_position.status.success());
    assert!(stderr(&horizon_position).contains("horizon rows do not accept --position"));
}

#[test]
fn id_completed_and_unknown_remove_fields_are_not_editable() {
    let dir = LedgerDir::empty();
    dir.write(common::MINIMAL);
    let path = dir.path.to_string_lossy();
    for flag in ["--id", "--completed"] {
        let output = qctl(&["edit", "QCTL-001", "-f", &path, flag, "value"]);
        assert!(!output.status.success());
        assert!(stderr(&output).contains("unexpected argument"));
    }
    let remove = qctl(&["edit", "QCTL-001", "-f", &path, "-x", "title:value"]);
    assert!(!remove.status.success());
    assert!(stderr(&remove).contains("remove field must be"));
}
