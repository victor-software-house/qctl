//! What each verb does to a file, asserted as the whole file.
//!
//! One directory per case under `fixtures/mutations`, holding the ledger going
//! in, the argv, and the ledger that must come out. The assertion is byte
//! equality with `after.yaml`, so a verb that touched a line it had no business
//! touching fails even though the values are right — which is the whole point
//! of the writer. `TODAY` in an expected file stands for the current UTC date,
//! the one value a fixture cannot pin.

mod common;

use common::{LedgerDir, qctl, stderr};
use rstest::rstest;
use std::fs;
use std::path::{Path, PathBuf};

#[rstest]
fn a_verb_changes_only_what_it_must(
    #[files("tests/fixtures/mutations/*")]
    #[dirs]
    case: PathBuf,
) {
    let before = read(&case, "before.yaml");
    let expected = read(&case, "after.yaml").replace("TODAY", &today());

    let dir = LedgerDir::empty();
    dir.write(&before);
    let path = dir.path.to_string_lossy().into_owned();
    let argv: Vec<String> = read(&case, "command").lines().map(str::to_owned).collect();
    let mut args: Vec<&str> = argv.iter().map(String::as_str).collect();
    args.extend(["--file", &path]);

    let output = qctl(&args);
    assert!(
        output.status.success(),
        "{} refused the ledger:\n{}",
        argv.join(" "),
        stderr(&output)
    );
    assert_eq!(
        dir.read(),
        expected,
        "{} did not leave the file the fixture expects",
        argv.join(" ")
    );

    // Matching the fixture is not enough on its own: the file a verb leaves
    // behind has to be one the next verb, and every consumer, still accepts.
    let checked = qctl(&["check", "--file", &path, "--no-git"]);
    assert!(
        checked.status.success(),
        "check refuses what {} wrote:\n{}",
        argv.join(" "),
        stderr(&checked)
    );
}

/// The row this ledger's last queued id names, moved to the front — the longest
/// journey a move can make in it, and so the one with the most to disturb.
const MOVED: &str = "QCTL-009";

/// The fixtures pin small files exactly. This says what they cannot at size: on
/// a 239-line ledger with seventeen folded scalars, the row arrives whole and at
/// the front, once, and every other line is still where it was.
///
/// The ledger is a frozen copy rather than this repository's live one, which
/// would change what the test exercises every time a row is archived.
#[test]
fn a_row_moved_in_a_real_ledger_arrives_whole_and_disturbs_nothing() {
    let before = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/a-real-ledger.yaml"),
    )
    .expect("the frozen ledger");
    let row = rows_of(&before, MOVED);

    let dir = LedgerDir::empty();
    dir.write(&before);
    let path = dir.path.to_string_lossy().into_owned();
    let output = qctl(&["start", MOVED, "--file", &path]);
    assert!(output.status.success(), "{}", stderr(&output));
    let after = dir.read();

    let block = row.join("\n");
    assert!(
        after.contains(&format!("queue:\n{block}\n")),
        "{MOVED} did not arrive whole at the front of the queue"
    );
    assert_eq!(
        after.matches(&block).count(),
        1,
        "{MOVED} appears more than once"
    );

    // In order, so a row that landed somewhere unexpected, or two rows that
    // swapped, is a failure rather than the same lines in a different sequence.
    assert_eq!(
        kept(&before, &row),
        kept(&after, &row),
        "moving {MOVED} disturbed a line that was not its own"
    );
    assert!(
        after.contains(&format!("active: {MOVED}")),
        "active is stale"
    );
}

/// The lines of the row this id names, from `- id:` to the line before the next
/// row or section.
fn rows_of(ledger: &str, id: &str) -> Vec<String> {
    let lines: Vec<&str> = ledger.lines().collect();
    let start = lines
        .iter()
        .position(|line| line.trim_start() == format!("- id: {id}"))
        .expect("the row is in this ledger");
    let depth = lines[start].len() - lines[start].trim_start().len();
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find(|(_, line)| !line.trim().is_empty() && line.len() - line.trim_start().len() <= depth)
        .map_or(lines.len(), |(at, _)| at);
    lines[start..end]
        .iter()
        .map(|line| (*line).to_owned())
        .filter(|line| !line.trim().is_empty())
        .collect()
}

/// Everything the move was not about: every line except the row's own, the
/// `active` line, and the blank lines the fixtures already pin exactly.
fn kept(ledger: &str, row: &[String]) -> Vec<String> {
    ledger
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with("active:"))
        .map(str::to_owned)
        .filter(|line| !row.contains(line))
        .collect()
}

fn read(case: &Path, name: &str) -> String {
    fs::read_to_string(case.join(name))
        .unwrap_or_else(|error| panic!("{}: {error}", case.join(name).display()))
}

fn today() -> String {
    time::OffsetDateTime::now_utc()
        .date()
        .format(time::macros::format_description!("[year]-[month]-[day]"))
        .expect("format today")
}
