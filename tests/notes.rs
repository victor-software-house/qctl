mod common;

use common::{LedgerDir, qctl, stderr};
use indoc::indoc;

#[test]
fn repeated_notes_are_readable_distinct_list_items() {
    let dir = LedgerDir::empty();
    dir.write(common::MINIMAL);
    let path = dir.path.to_string_lossy();
    let output = qctl(&[
        "add",
        "-f",
        &path,
        "-t",
        "t",
        "-s",
        "s",
        "-o",
        "o",
        "-a",
        "a",
        "-n",
        "Short context.",
        "-n",
        "Source: the colon stays readable.",
        "-n",
        "First intentional line.\nSecond intentional line.\n\nNext paragraph.",
        "-n",
        "  Indented first line.\nPlain second line.",
    ]);
    assert!(output.status.success(), "{}", stderr(&output));
    let body = dir.read();
    assert!(
        body.contains(
            "    notes:\n      - Short context.\n      - >2-\n        Source: the colon stays readable.\n      - |2-\n        First intentional line.\n        Second intentional line.\n\n        Next paragraph.\n      - |2-\n          Indented first line.\n        Plain second line."
        ),
        "{body}"
    );
    let ledger: serde_yml::Value = serde_yml::from_str(&body).expect("parse output");
    let notes = ledger["queue"][0]["notes"]
        .as_sequence()
        .expect("notes list");
    assert_eq!(notes.len(), 4);
    assert_eq!(
        notes[2].as_str(),
        Some("First intentional line.\nSecond intentional line.\n\nNext paragraph.")
    );
    assert_eq!(
        notes[3].as_str(),
        Some("  Indented first line.\nPlain second line.")
    );
}

#[test]
fn edit_uses_the_same_note_item_policy() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {"
        schema_version: 4
        prefix: QCTL
        active: null
        queue:
          - id: QCTL-001
            title: t
            scope: s
            outcome: o
            blocked_by: []
            acceptance: [a]
            notes: [Existing.]
        archive: []
        horizon: []
    "});
    let path = dir.path.to_string_lossy();
    let output = qctl(&[
        "edit",
        "QCTL-001",
        "-f",
        &path,
        "-n",
        "Context: added.",
        "-n",
        "Line one.\nLine two.",
    ]);
    assert!(output.status.success(), "{}", stderr(&output));
    let body = dir.read();
    assert!(body.contains("- Existing."), "{body}");
    assert!(body.contains("- >2-\n        Context: added."), "{body}");
    assert!(
        body.contains("- |2-\n        Line one.\n        Line two."),
        "{body}"
    );
}
