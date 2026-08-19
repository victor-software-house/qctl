//! What each verb does to a file, as one reviewed record per scenario.
//!
//! A case is a directory under `fixtures/mutations` holding the two things a
//! person writes: `before.yaml`, the ledger going in, and `command`, the line
//! they would type. What comes out is not written by hand. It is recorded as
//! the whole resulting file with every line marked `-` removed, `+` added, or
//! unchanged — so one artifact is both the exact result and the diff that
//! explains it, and a scenario is read once rather than twice.
//!
//! The count at the top says how many lines moved before anything is scrolled.
//! Any change to what a verb writes changes it, and `cargo insta review` shows
//! the snapshot's own before and after when the change is intended.

mod common;

use common::{LedgerDir, qctl, stderr};
use rstest::rstest;
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::{Path, PathBuf};

#[rstest]
fn a_verb_changes_only_what_it_must(
    #[files("tests/fixtures/mutations/*")]
    #[dirs]
    case: PathBuf,
) {
    let name = case
        .file_name()
        .and_then(|name| name.to_str())
        .expect("a named case");
    let before = read(&case, "before.yaml");
    let line = read(&case, "command");
    let line = line.trim();
    let argv = shlex::split(line).unwrap_or_else(|| panic!("{name}: {line} is not a command"));

    let dir = LedgerDir::empty();
    dir.write(&before);
    let path = dir.path.to_string_lossy().into_owned();
    let mut args: Vec<&str> = argv.iter().map(String::as_str).collect();
    args.extend(["--file", &path]);

    let output = qctl(&args);
    assert!(
        output.status.success(),
        "qctl {line} refused the ledger:\n{}",
        stderr(&output)
    );
    let after = dir.read();

    // The file a verb leaves behind has to be one the next verb, and every
    // consumer, still accepts. A snapshot on its own would happily record a
    // ledger that no longer loads.
    let checked = qctl(&["check", "--file", &path, "--no-git"]);
    assert!(
        checked.status.success(),
        "check refuses what qctl {line} wrote:\n{}",
        stderr(&checked)
    );

    // The instant `archive` stamps is the one value a case cannot pin, so it is
    // the one held loosely — on a line that verb wrote, in a case that runs it.
    // `fmt` only ever moves a stamp somebody else wrote, and a filter that
    // could not tell the difference would stop asserting the value it moved.
    let stamped = argv.first().is_some_and(|verb| verb == "archive");
    insta::with_settings!({
        filters => if stamped {
            vec![(r"(?m)^(\+\s*completed: )\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}$", "${1}[NOW]")]
        } else {
            Vec::new()
        },
        description => format!("qctl {line}"),
        omit_expression => true,
    }, {
        insta::assert_snapshot!(name, marked(&before, &after));
    });
}

/// The whole file a verb wrote, each line marked with what happened to it.
fn marked(before: &str, after: &str) -> String {
    let diff = TextDiff::from_lines(before, after);
    let mut body = String::new();
    let (mut removed, mut added) = (0, 0);
    for change in diff.iter_all_changes() {
        let mark = match change.tag() {
            ChangeTag::Delete => {
                removed += 1;
                '-'
            }
            ChangeTag::Insert => {
                added += 1;
                '+'
            }
            ChangeTag::Equal => ' ',
        };
        body.push(mark);
        body.push_str(change.value());
    }
    format!("# -{removed} +{added}\n{body}")
}

fn read(case: &Path, name: &str) -> String {
    fs::read_to_string(case.join(name))
        .unwrap_or_else(|error| panic!("{}: {error}", case.join(name).display()))
}
