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

/// The fixtures pin small files exactly. This one takes the real ledger of this
/// repository — hundreds of lines, comments, folded scalars — and says the thing
/// the fixtures cannot say at that size: every line outside the row that moved,
/// and the `active` line, is the line it was.
#[test]
fn moving_a_row_in_a_real_ledger_leaves_every_other_line_alone() {
    let before = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tasks.yaml"))
        .expect("this repository's own ledger");

    let dir = LedgerDir::empty();
    dir.write(&before);
    let path = dir.path.to_string_lossy().into_owned();
    let moved = last_queued_id(&before);
    let output = qctl(&["start", &moved, "--file", &path]);
    assert!(output.status.success(), "{}", stderr(&output));
    let after = dir.read();

    let untouched = |body: &str| -> Vec<String> {
        body.lines()
            .filter(|line| !line.starts_with("active:"))
            .map(str::to_owned)
            .collect()
    };
    let mut was = untouched(&before);
    let mut is = untouched(&after);
    was.sort();
    is.sort();
    assert_eq!(
        was, is,
        "moving {moved} changed a line that was not the row moving or `active`"
    );
    assert_ne!(before, after, "nothing moved at all");
}

/// The id of the last row on the queue, which is the furthest one a move can
/// travel and so the one that would disturb the most. A ledger writes its
/// sections in whatever order it likes, so the queue ends at the next key.
fn last_queued_id(ledger: &str) -> String {
    ledger
        .lines()
        .skip_while(|line| !line.starts_with("queue:"))
        .skip(1)
        .take_while(|line| line.starts_with([' ', '#']) || line.trim().is_empty())
        .filter_map(|line| line.trim_start().strip_prefix("- id:"))
        .last()
        .expect("a queued row")
        .trim()
        .trim_matches('"')
        .to_owned()
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
