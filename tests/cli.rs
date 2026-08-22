//! Binary contract: init, check, add, start, archive, park, promote.

mod common;

use common::{LedgerDir, MINIMAL, qctl, stderr, stdout};
use indoc::indoc;

#[test]
fn init_writes_ledger_without_schema_copy() {
    let dir = LedgerDir::empty();
    let output = qctl(&["init", "-p", "omx", "-f", dir.path.to_str().unwrap()]);
    assert!(output.status.success(), "{}", stderr(&output));
    let body = dir.read();
    assert!(body.contains("prefix: OMX"));
    assert!(body.contains("horizon: []"));
    assert!(body.contains("$schema="));
    assert!(!dir.parent().join("tasks.schema.json").exists());
}

#[test]
fn init_refuses_overwrite_without_force() {
    let dir = LedgerDir::empty();
    dir.write(MINIMAL);
    let output = qctl(&["init", "-p", "QCTL", "-f", dir.path.to_str().unwrap()]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("already exists"));
}

#[test]
fn check_accepts_own_repo_shape() {
    let dir = LedgerDir::empty();
    dir.write(MINIMAL);
    let output = qctl(&["check", "-f", dir.path.to_str().unwrap()]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains("ok"));
}

#[test]
fn check_accepts_archive_notes() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {"
        schema_version: 3
        prefix: QCTL
        active: null
        queue: []
        archive:
          - id: QCTL-001
            title: done
            scope: s
            completed: 2026-08-17T09:12:00
            outcome: o
            evidence: [landed]
            notes: >-
              Keep the context that would not fit in evidence.
    "});
    let output = qctl(&["check", "-f", dir.path.to_str().unwrap()]);
    assert!(output.status.success(), "{}", stderr(&output));
}

/// The ledger declares its zone, so a stamp is already local and `show` must not
/// shift it — only separate the day from the time. This stamp sits late in the
/// evening on purpose: a version that converted it, in either direction, would
/// name a different day and not merely a different hour.
#[test]
fn show_reads_a_stamp_without_moving_it() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {"
        schema_version: 3
        prefix: QCTL
        style:
          timezone: \"-03:00\"
        active: null
        queue: []
        archive:
          - id: QCTL-001
            title: Shipped late
            scope: qctl
            completed: 2026-08-16T23:08:28
            outcome: It shipped.
            evidence: [The tag exists.]
    "});
    let output = qctl(&["show", "QCTL-001", "-f", dir.path.to_str().unwrap()]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        stdout(&output).trim_end(),
        "QCTL-001  Shipped late  (archived 2026-08-16 23:08:28)"
    );
}

/// The zone a stamp is written in comes from the ledger, and the only way to see
/// that is to read the stamp back as the declared zone and land on now. Written
/// in UTC instead, it would be three hours out.
#[test]
fn archive_stamps_in_the_zone_the_ledger_declares() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {"
        schema_version: 3
        prefix: QCTL
        style:
          timezone: \"-03:00\"
        active: QCTL-001
        queue:
          - id: QCTL-001
            title: About to close
            scope: qctl
            outcome: Something is true.
            blocked_by: []
            acceptance: [It holds.]
        archive: []
    "});
    let archived = qctl(&[
        "archive",
        "QCTL-001",
        "-f",
        dir.path.to_str().unwrap(),
        "-e",
        "It shipped.",
    ]);
    assert!(archived.status.success(), "{}", stderr(&archived));

    let body = dir.read();
    let stamp = body
        .lines()
        .find_map(|line| line.trim().strip_prefix("completed: "))
        .expect("a stamp");
    let written = time::PrimitiveDateTime::parse(
        stamp,
        time::macros::format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]"),
    )
    .unwrap_or_else(|error| panic!("{stamp}: {error}"))
    .assume_offset(time::macros::offset!(-3));
    let drift = time::OffsetDateTime::now_utc() - written;
    assert!(
        drift.abs() < time::Duration::seconds(30),
        "{stamp} read as -03:00 is {drift} from now, so it was not written in that zone"
    );
}

/// `fmt --check` is for a hook: it writes nothing, exits non-zero, and says
/// which line to look at rather than only that something is out of style.
#[test]
fn fmt_check_names_the_line_and_writes_nothing() {
    let dir = LedgerDir::empty();
    let untidy = indoc! {"
        schema_version: 3
        prefix: QCTL
        active: null
        queue: []
        archive: []
        horizon: []


    "};
    dir.write(untidy);
    let output = qctl(&["fmt", "--check", "-f", dir.path.to_str().unwrap()]);
    assert!(!output.status.success(), "accepted an untidy ledger");
    assert!(
        stderr(&output).contains("is not in its declared style"),
        "{}",
        stderr(&output)
    );
    assert_eq!(dir.read(), untidy, "--check wrote to the file");

    let fixed = qctl(&["fmt", "-f", dir.path.to_str().unwrap()]);
    assert!(fixed.status.success(), "{}", stderr(&fixed));
    assert!(dir.read().ends_with("horizon: []\n"));
    assert!(
        qctl(&["fmt", "--check", "-f", dir.path.to_str().unwrap()])
            .status
            .success()
    );
}

/// A comment directly above a key belongs to that list and moves with it. One
/// with a blank line under it belongs to nothing, and reordering around it would
/// either carry it to the wrong place or lose it. `fmt` refuses instead, and
/// leaves the file exactly as it was.
#[test]
fn fmt_refuses_to_move_lists_around_a_comment_it_cannot_place() {
    let dir = LedgerDir::empty();
    let orphaned = indoc! {"
        schema_version: 3
        prefix: QCTL
        style:
          section_order: [queue, horizon, archive]
        active: null
        queue: []

        # A note with a blank line under it, so it sits between the lists.

        archive: []
        horizon: []
    "};
    dir.write(orphaned);
    let output = qctl(&["fmt", "-f", dir.path.to_str().unwrap()]);
    assert!(!output.status.success(), "reordered around a loose comment");
    assert!(
        stderr(&output).contains("belongs to neither"),
        "{}",
        stderr(&output)
    );
    assert_eq!(dir.read(), orphaned, "the file was written to anyway");
}

#[test]
fn check_rejects_unknown_field() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {"
        schema_version: 3
        prefix: QCTL
        active: null
        queue: []
        archive: []
        nope: true
    "});
    let output = qctl(&["check", "-f", dir.path.to_str().unwrap()]);
    assert!(!output.status.success());
}

#[test]
fn add_start_archive_round_trip() {
    let dir = LedgerDir::empty();
    assert!(
        qctl(&["init", "-p", "QCTL", "-f", dir.path.to_str().unwrap()])
            .status
            .success()
    );

    let add = qctl(&[
        "add",
        "-f",
        dir.path.to_str().unwrap(),
        "-t",
        "First",
        "-s",
        "qctl",
        "-o",
        "done",
        "-a",
        "shipped",
    ]);
    assert!(add.status.success(), "{}", stderr(&add));
    assert_eq!(stdout(&add).trim(), "QCTL-001");

    let start = qctl(&["start", "QCTL-001", "-f", dir.path.to_str().unwrap()]);
    assert!(start.status.success(), "{}", stderr(&start));

    let check = qctl(&["check", "-f", dir.path.to_str().unwrap()]);
    assert!(check.status.success(), "{}", stderr(&check));

    let archive = qctl(&[
        "archive",
        "QCTL-001",
        "-f",
        dir.path.to_str().unwrap(),
        "-e",
        "landed",
    ]);
    assert!(archive.status.success(), "{}", stderr(&archive));

    let after = qctl(&["check", "-f", dir.path.to_str().unwrap()]);
    assert!(after.status.success(), "{}", stderr(&after));
    let body = dir.read();
    assert!(body.contains("disposition: completed"));
    assert!(body.contains("QCTL-001"));
}

#[test]
fn add_writes_notes_blockers_plan_and_links() {
    let dir = LedgerDir::empty();
    assert!(
        qctl(&["init", "-p", "QCTL", "-f", dir.path.to_str().unwrap()])
            .status
            .success()
    );
    let first = qctl(&[
        "add",
        "-f",
        dir.path.to_str().unwrap(),
        "-t",
        "First",
        "-s",
        "qctl",
        "-o",
        "done",
        "-a",
        "shipped",
    ]);
    assert!(first.status.success(), "{}", stderr(&first));
    dir.plant("docs/plan.md", "# Plan\n");
    let second = qctl(&[
        "add",
        "-f",
        dir.path.to_str().unwrap(),
        "-t",
        "Second",
        "-s",
        "qctl",
        "-o",
        "done",
        "-a",
        "shipped",
        "--notes",
        "Why this exists.",
        "--blocked-by",
        "QCTL-001",
        "--plan",
        "docs/plan.md",
        "--link",
        "https://example.com/x",
        "--after",
        "QCTL-001",
    ]);
    assert!(second.status.success(), "{}", stderr(&second));
    let body = dir.read();
    assert!(body.contains("notes: Why this exists."), "{body}");
    assert!(
        body.contains("blocked_by:\n      - QCTL-001") || body.contains("blocked_by: [QCTL-001]"),
        "{body}"
    );
    assert!(body.contains("plan: docs/plan.md"), "{body}");
    assert!(body.contains("https://example.com/x"), "{body}");
    let check = qctl(&["check", "-f", dir.path.to_str().unwrap(), "--no-git"]);
    assert!(check.status.success(), "{}", stderr(&check));
}

#[test]
fn add_refuses_a_blocker_that_would_not_sit_earlier() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {"
        schema_version: 3
        prefix: QCTL
        active: QCTL-001
        queue:
          - id: QCTL-001
            title: First
            scope: s
            outcome: o
            blocked_by: []
            acceptance: [It holds.]
          - id: QCTL-002
            title: Second
            scope: s
            outcome: o
            blocked_by: []
            acceptance: [It holds.]
        archive: []
        horizon: []
    "});
    let output = qctl(&[
        "add",
        "-f",
        dir.path.to_str().unwrap(),
        "-t",
        "Third",
        "-s",
        "s",
        "-o",
        "o",
        "-a",
        "a",
        "--before",
        "QCTL-002",
        "--blocked-by",
        "QCTL-002",
    ]);
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("is not earlier"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn add_refuses_to_become_queue_head_while_active_is_set() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {"
        schema_version: 3
        prefix: QCTL
        active: QCTL-001
        queue:
          - id: QCTL-001
            title: First
            scope: s
            outcome: o
            blocked_by: []
            acceptance: [It holds.]
        archive: []
        horizon: []
    "});
    let output = qctl(&[
        "add",
        "-f",
        dir.path.to_str().unwrap(),
        "-t",
        "Ahead",
        "-s",
        "s",
        "-o",
        "o",
        "-a",
        "a",
        "--before",
        "QCTL-001",
    ]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("queue[0]"), "{}", stderr(&output));
}

#[test]
fn promote_refuses_a_queued_id() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {"
        schema_version: 3
        prefix: QCTL
        active: QCTL-001
        queue:
          - id: QCTL-001
            title: First
            scope: s
            outcome: o
            blocked_by: []
            acceptance: [It holds.]
        archive: []
        horizon: []
    "});
    let output = qctl(&[
        "promote",
        "QCTL-001",
        "-f",
        dir.path.to_str().unwrap(),
        "-a",
        "a",
    ]);
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("not on the horizon"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn promote_keeps_active_on_the_queue_head() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {"
        schema_version: 3
        prefix: QCTL
        active: QCTL-001
        queue:
          - id: QCTL-001
            title: First
            scope: s
            outcome: o
            blocked_by: []
            acceptance: [It holds.]
        archive: []
        horizon:
          - id: QCTL-002
            title: Parked
            scope: s
            outcome: o
            kind: research
            open: missing
    "});
    let output = qctl(&[
        "promote",
        "QCTL-002",
        "-f",
        dir.path.to_str().unwrap(),
        "-a",
        "It holds.",
    ]);
    assert!(output.status.success(), "{}", stderr(&output));
    let body = dir.read();
    assert!(body.contains("active: QCTL-001"), "{body}");
    assert!(!body.contains("kind: research"), "{body}");
    let check = qctl(&["check", "-f", dir.path.to_str().unwrap(), "--no-git"]);
    assert!(check.status.success(), "{}", stderr(&check));
}

#[test]
fn start_refuses_blocked_or_horizon() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {"
        schema_version: 3
        prefix: QCTL
        active: null
        queue:
          - id: QCTL-002
            title: blocked
            scope: s
            outcome: o
            blocked_by: [QCTL-001]
            acceptance: [a]
          - id: QCTL-001
            title: first
            scope: s
            outcome: o
            blocked_by: []
            acceptance: [a]
        archive: []
        horizon:
          - id: QCTL-009
            title: later
            scope: s
            outcome: o
            kind: research
            open: unknown
    "});
    let blocked = qctl(&["start", "QCTL-002", "-f", dir.path.to_str().unwrap()]);
    assert!(!blocked.status.success());
    assert!(stderr(&blocked).contains("blocked"));

    let horizon = qctl(&["start", "QCTL-009", "-f", dir.path.to_str().unwrap()]);
    assert!(!horizon.status.success());
    assert!(stderr(&horizon).contains("not queued"));
}

#[test]
fn instructions_prints_contract() {
    let output = qctl(&["instructions"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("horizon"));
    assert!(text.contains("tasks.yaml"));
}

#[test]
fn status_lists_horizon() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {"
        schema_version: 3
        prefix: QCTL
        active: QCTL-001
        queue:
          - id: QCTL-001
            title: now
            scope: s
            outcome: o
            blocked_by: []
            acceptance: [a]
        archive: []
        horizon:
          - id: QCTL-008
            title: later
            scope: s
            outcome: o
            kind: evaluation
            open: wait
    "});
    let output = qctl(&["status", "-f", dir.path.to_str().unwrap()]);
    assert!(output.status.success(), "{}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("QCTL-001"));
    assert!(text.contains("QCTL-008"));
    assert!(text.contains("evaluation"));
}

#[test]
fn park_writes_a_missing_horizon_key() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {"
        schema_version: 3
        prefix: QCTL
        active: null
        queue: []
        archive: []
    "});
    let output = qctl(&[
        "park",
        "-f",
        dir.path.to_str().unwrap(),
        "-t",
        "later",
        "-s",
        "s",
        "-o",
        "o",
        "--kind",
        "research",
        "--open",
        "wait",
    ]);
    assert!(output.status.success(), "{}", stderr(&output));
    let body = dir.read();
    assert!(body.contains("horizon:"), "{body}");
    assert!(body.contains("QCTL-001"), "{body}");
}

#[test]
fn add_refuses_a_plan_that_is_not_a_file() {
    let dir = LedgerDir::empty();
    dir.write(MINIMAL);
    let output = qctl(&[
        "add",
        "-f",
        dir.path.to_str().unwrap(),
        "-t",
        "t",
        "-s",
        "s",
        "-o",
        "o",
        "-a",
        "a",
        "--plan",
        "docs/nope.md",
    ]);
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("missing plan docs/nope.md"),
        "{}",
        stderr(&output)
    );
    assert_eq!(dir.read(), MINIMAL);
}

#[test]
fn add_to_an_empty_queue_does_not_invent_before() {
    let dir = LedgerDir::empty();
    dir.write(indoc! {"
        schema_version: 3
        prefix: QCTL
        active: QCTL-099
        queue: []
        archive: []
        horizon: []
    "});
    let output = qctl(&[
        "add",
        "-f",
        dir.path.to_str().unwrap(),
        "-t",
        "t",
        "-s",
        "s",
        "-o",
        "o",
        "-a",
        "a",
    ]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(!stderr(&output).contains("--before"), "{}", stderr(&output));
    assert!(dir.read().contains("QCTL-001"));
}
