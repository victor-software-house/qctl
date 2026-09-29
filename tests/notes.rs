mod common;

use common::{LedgerDir, qctl, stderr};
use ctl_core::input::Input;
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
            "    notes:\n      - Short context.\n      - >-\n        Source: the colon stays readable.\n      - |-\n        First intentional line.\n        Second intentional line.\n\n        Next paragraph.\n      - |2-\n          Indented first line.\n        Plain second line."
        ),
        "{body}"
    );
    let ledger: serde_json::Value = Input::new("tasks.yaml", body.as_str())
        .parse()
        .expect("parse output");
    let notes = ledger["queue"][0]["notes"].as_array().expect("notes list");
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
fn trailing_and_blank_lines_survive_fmt() {
    let dir = LedgerDir::empty();
    dir.write(common::MINIMAL);
    let path = dir.path.to_string_lossy();
    let notes = ["one trailing\n", "two trailing\n\n", "first\n\n\n  \nlast"];
    let output = qctl(&[
        "add", "-f", &path, "-t", "t", "-s", "s", "-o", "o", "-a", "a", "-n", notes[0], "-n",
        notes[1], "-n", notes[2],
    ]);
    assert!(output.status.success(), "{}", stderr(&output));
    let before = parsed_notes(&dir.read());
    assert_eq!(before, notes);

    let formatted = qctl(&["fmt", "-f", &path]);
    assert!(formatted.status.success(), "{}", stderr(&formatted));
    assert_eq!(parsed_notes(&dir.read()), notes);
}

fn parsed_notes(body: &str) -> Vec<String> {
    let ledger: serde_json::Value = Input::new("tasks.yaml", body)
        .parse()
        .expect("parse output");
    ledger["queue"][0]["notes"]
        .as_array()
        .expect("notes list")
        .iter()
        .map(|note| note.as_str().expect("note string").to_owned())
        .collect()
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
    assert!(body.contains("- >-\n        Context: added."), "{body}");
    assert!(
        body.contains("- |-\n        Line one.\n        Line two."),
        "{body}"
    );
}

/// A note that ends in two line breaks is written with keep chomping (`|+`),
/// and a row whose last note is one still moves and formats like any other.
#[test]
fn a_row_whose_last_note_keeps_its_line_breaks_still_moves() {
    let dir = LedgerDir::empty();
    dir.write(common::MINIMAL);
    let path = dir.path.to_string_lossy();
    for title in ["first", "second"] {
        let output = qctl(&[
            "add", "-f", &path, "-t", title, "-s", "s", "-o", "o", "-a", "a", "-n", "ctx\n\n",
        ]);
        assert!(output.status.success(), "{}", stderr(&output));
    }
    for args in [
        vec!["start", "QCTL-002"],
        vec!["fmt"],
        vec!["archive", "QCTL-002", "-e", "done"],
    ] {
        let mut argv = args.clone();
        argv.extend(["-f", &path]);
        let output = qctl(&argv);
        assert!(output.status.success(), "{args:?}: {}", stderr(&output));
    }
    let ledger: serde_json::Value = Input::new("tasks.yaml", dir.read().as_str())
        .parse()
        .expect("parse output");
    assert_eq!(ledger["queue"][0]["notes"][0], "ctx\n\n");
    assert_eq!(ledger["archive"][0]["notes"][0], "ctx\n\n");
}
