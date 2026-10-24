//! List items through `add`, `edit`, and `fmt`, in each declared layout
//! and quotes.

mod common;

use common::{LedgerDir, qctl, stderr};
use ctl_core::input::Input;
use indoc::indoc;

#[test]
fn every_note_is_written_on_its_own_line() {
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
        "Done: every row passes",
        "-n",
        "Short context.",
        "-n",
        "2026-10-15: found in review",
        "-n",
        "2026-10-15, found in review",
        "-n",
        "2026-10-15: it's blocked on review",
        "-n",
        "Done: every row passes",
    ]);
    assert!(output.status.success(), "{}", stderr(&output));
    let body = dir.read();
    let written: Vec<&str> = body
        .lines()
        .skip_while(|line| line.trim_start() != "acceptance:")
        .collect();
    let expected = [
        "    acceptance:",
        "      - 'Done: every row passes'",
        "    notes:",
        "      - Short context.",
        "      - '2026-10-15: found in review'",
        "      - 2026-10-15, found in review",
        r#"      - "2026-10-15: it's blocked on review""#,
        "      - 'Done: every row passes'",
    ];
    assert_eq!(
        written.get(..expected.len()),
        Some(expected.as_slice()),
        "{body}"
    );
    assert!(!body.contains("''"), "{body}");
}

#[test]
fn a_note_holding_a_line_break_is_refused() {
    let dir = LedgerDir::empty();
    let ledger = common::queued(QUEUED);
    dir.write(&ledger);
    let path = dir.path.to_string_lossy();
    for note in ["First line.\nSecond line.", "One line.\r"] {
        for args in [
            [
                "add", "-t", "t", "-s", "s", "-o", "o", "-a", "a", "-n", note,
            ]
            .as_slice(),
            ["edit", "QCTL-001", "-n", note].as_slice(),
        ] {
            let mut argv = args.to_vec();
            argv.extend(["-f", &path]);
            let output = qctl(&argv);
            assert!(!output.status.success(), "{args:?} was accepted");
            let error = stderr(&output);
            assert!(
                error.contains("a note is one line: pass each line as its own -n/--note"),
                "{error}"
            );
            assert_eq!(dir.read(), ledger);
        }
    }
}

/// Under `list_items: auto`, a note keeps its line breaks as a literal block,
/// and `fmt` leaves it that way.
#[test]
fn the_automatic_layout_keeps_line_breaks() {
    let dir = LedgerDir::empty();
    dir.write(AUTO);
    let path = dir.path.to_string_lossy();
    let notes = [
        "Short context.",
        "Source: the colon stays readable.",
        "First intentional line.\nSecond intentional line.\n\nNext paragraph.",
        "  Indented first line.\nPlain second line.",
        "one trailing\n",
        "two trailing\n\n",
    ];
    let mut argv = vec![
        "add", "-f", &path, "-t", "t", "-s", "s", "-o", "o", "-a", "a",
    ];
    argv.extend(notes.iter().flat_map(|note| ["-n", note]));
    let output = qctl(&argv);
    assert!(output.status.success(), "{}", stderr(&output));
    let body = dir.read();
    let written: Vec<&str> = body
        .lines()
        .skip_while(|line| line.trim_start() != "notes:")
        .collect();
    let expected = [
        "    notes:",
        "      - Short context.",
        "      - 'Source: the colon stays readable.'",
        "      - |-",
        "        First intentional line.",
        "        Second intentional line.",
        "",
        "        Next paragraph.",
        "      - |2-",
        "          Indented first line.",
        "        Plain second line.",
    ];
    assert_eq!(
        written.get(..expected.len()),
        Some(expected.as_slice()),
        "{body}"
    );
    assert_eq!(strings(&row(&body)["notes"]), notes);

    let formatted = qctl(&["fmt", "-f", &path]);
    assert!(formatted.status.success(), "{}", stderr(&formatted));
    assert_eq!(dir.read(), body);
}

#[test]
fn single_quotes_when_the_ledger_asks() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {"
        schema_version: 5
        prefix: QCTL
        style:
          quotes: single
        active: null
        queue:
          - id: QCTL-001
            title: t
            scope: s
            outcome: o
            blocked_by: []
            acceptance:
              - a
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
        "2026-10-15: it's blocked",
    ]);
    assert!(output.status.success(), "{}", stderr(&output));
    let body = dir.read();
    assert!(
        has_line(&body, "      - '2026-10-15: it''s blocked'"),
        "{body}"
    );
}

#[test]
fn edit_uses_the_same_item_writer() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {"
        schema_version: 5
        prefix: QCTL
        active: null
        queue:
          - id: QCTL-001
            title: t
            scope: s
            outcome: o
            blocked_by: []
            acceptance:
              - a
            notes:
              - Existing.
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
        "-a",
        "It's checked: yes",
    ]);
    assert!(output.status.success(), "{}", stderr(&output));
    let expected = indoc! {r#"
        schema_version: 5
        prefix: QCTL
        active: null
        queue:
          - id: QCTL-001
            title: t
            scope: s
            outcome: o
            blocked_by: []
            acceptance:
              - a
              - "It's checked: yes"
            notes:
              - Existing.
              - 'Context: added.'
        archive: []
        horizon: []
    "#};
    assert_eq!(dir.read(), expected);
}

/// `fmt` writes every block item on one line and every string in the
/// declared quotes, a second run finds nothing to change, and each value
/// reads back as it did, a multi-line note as its lines.
#[test]
fn fmt_keeps_every_value_and_settles() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {r#"
        schema_version: 5
        prefix: "QCTL"
        active: QCTL-001
        queue:
          - id: QCTL-001
            title: 'It''s here: now'
            scope: "s"
            outcome: o
            blocked_by: []
            acceptance:
              - >-
                The first half,
                and the second half.
            notes:
              - >2-
                2026-09-28: the old shape
              - |-
                Lead: the design.

                1. It's the first.
              - "First.\nSecond."
              - |+
                Kept: a final line break.

        archive: []
        horizon: []
    "#});
    let path = dir.path.to_string_lossy();
    let before = row(&dir.read());

    let formatted = qctl(&["fmt", "-f", &path]);
    assert!(formatted.status.success(), "{}", stderr(&formatted));
    let once = dir.read();
    let after = row(&once);
    for key in ["title", "scope", "acceptance"] {
        assert_eq!(after[key], before[key], "{key}");
    }
    for line in [
        r#"    title: "It's here: now""#,
        "prefix: QCTL",
        "    scope: s",
    ] {
        assert!(has_line(&once, line), "{once}");
    }
    let expected_notes: Vec<String> = strings(&before["notes"])
        .iter()
        .flat_map(|note| note.split('\n'))
        .filter(|line| !line.trim().is_empty())
        .map(ToOwned::to_owned)
        .collect();
    assert_eq!(strings(&after["notes"]), expected_notes);
    assert!(
        !once.lines().any(|line| {
            let item = line.trim_start();
            item.starts_with("- >") || item.starts_with("- |")
        }),
        "{once}"
    );

    let again = qctl(&["fmt", "--check", "-f", &path]);
    assert!(again.status.success(), "{}", stderr(&again));
    assert_eq!(dir.read(), once);
}

fn has_line(body: &str, wanted: &str) -> bool {
    body.lines().any(|line| line == wanted)
}

#[expect(
    clippy::expect_used,
    reason = "output that does not parse fails its test"
)]
fn row(body: &str) -> serde_json::Value {
    let ledger: serde_json::Value = Input::new("tasks.yaml", body)
        .parse()
        .expect("parse output");
    ledger["queue"][0].clone()
}

#[expect(
    clippy::expect_used,
    reason = "a list that is not strings fails its test"
)]
fn strings(list: &serde_json::Value) -> Vec<String> {
    list.as_array()
        .expect("a list")
        .iter()
        .map(|item| item.as_str().expect("a string").to_owned())
        .collect()
}

/// One queued row, `QCTL-001`.
const QUEUED: &str = indoc! {"
    - id: QCTL-001
      title: t
      scope: s
      outcome: o
      blocked_by: []
      acceptance:
        - a
"};

/// An empty ledger that writes its list items in the automatic layout.
const AUTO: &str = indoc! {"
    schema_version: 5
    prefix: QCTL
    style:
      list_items: auto
    active: null
    queue: []
    archive: []
    horizon: []
"};
