//! Graph rules that JSON Schema cannot express.

use qctl::ledger::{graph_errors, load};
use std::fs;
use tempfile::TempDir;

fn errors(body: &str) -> Vec<String> {
    let root = TempDir::new().expect("tempdir");
    let path = root.path().join("tasks.yaml");
    fs::write(&path, body).expect("write");
    let ledger = load(&path).expect("parse");
    graph_errors(&ledger, &path)
}

#[test]
fn empty_ledger_is_clean() {
    assert_eq!(
        errors(
            "\
schema_version: 1
prefix: QCTL
active: null
queue: []
archive: []
"
        ),
        Vec::<String>::new()
    );
}

#[test]
fn missing_horizon_defaults_to_empty() {
    let root = TempDir::new().expect("tempdir");
    let path = root.path().join("tasks.yaml");
    fs::write(
        &path,
        "\
schema_version: 1
prefix: PST
active: null
queue: []
archive: []
",
    )
    .expect("write");
    let ledger = load(&path).expect("parse");
    assert!(ledger.horizon.is_empty());
}

#[test]
fn rejects_wrong_prefix_and_short_id() {
    let found = errors(
        "\
schema_version: 1
prefix: QCTL
active: null
queue:
  - id: OMX-001
    title: t
    scope: s
    outcome: o
    blocked_by: []
    acceptance: [a]
  - id: QCTL-01
    title: t
    scope: s
    outcome: o
    blocked_by: []
    acceptance: [a]
archive: []
",
    );
    assert!(found.iter().any(|e| e.contains("OMX-001")));
    assert!(found.iter().any(|e| e.contains("QCTL-01")));
}

#[test]
fn rejects_duplicate_ids_across_lists() {
    let found = errors(
        "\
schema_version: 1
prefix: QCTL
active: null
queue:
  - id: QCTL-001
    title: t
    scope: s
    outcome: o
    blocked_by: []
    acceptance: [a]
archive:
  - id: QCTL-001
    title: t
    scope: s
    completed: '2026-08-16'
    outcome: o
    evidence: [e]
horizon:
  - id: QCTL-002
    title: t
    scope: s
    outcome: o
    kind: research
    open: why
",
    );
    assert!(found.iter().any(|e| e.contains("duplicate id QCTL-001")));
}

#[test]
fn rejects_active_that_is_not_queue_head() {
    let found = errors(
        "\
schema_version: 1
prefix: QCTL
active: QCTL-002
queue:
  - id: QCTL-001
    title: t
    scope: s
    outcome: o
    blocked_by: []
    acceptance: [a]
  - id: QCTL-002
    title: t
    scope: s
    outcome: o
    blocked_by: []
    acceptance: [a]
archive: []
",
    );
    assert!(found.iter().any(|e| e.contains("must be queue[0]")));
}

#[test]
fn rejects_blocked_active() {
    let found = errors(
        "\
schema_version: 1
prefix: QCTL
active: QCTL-002
queue:
  - id: QCTL-002
    title: t
    scope: s
    outcome: o
    blocked_by: [QCTL-001]
    acceptance: [a]
  - id: QCTL-001
    title: t
    scope: s
    outcome: o
    blocked_by: []
    acceptance: [a]
archive: []
",
    );
    assert!(found.iter().any(|e| e.contains("is blocked")));
}

#[test]
fn rejects_horizon_active() {
    let found = errors(
        "\
schema_version: 1
prefix: QCTL
active: QCTL-009
queue: []
archive: []
horizon:
  - id: QCTL-009
    title: later
    scope: s
    outcome: o
    kind: evaluation
    open: no start
",
    );
    assert!(found.iter().any(|e| e.contains("horizon")));
}

#[test]
fn rejects_blocker_not_earlier_or_missing() {
    let found = errors(
        "\
schema_version: 1
prefix: QCTL
active: null
queue:
  - id: QCTL-001
    title: t
    scope: s
    outcome: o
    blocked_by: [QCTL-002]
    acceptance: [a]
  - id: QCTL-002
    title: t
    scope: s
    outcome: o
    blocked_by: [QCTL-099]
    acceptance: [a]
archive: []
",
    );
    assert!(found.iter().any(|e| e.contains("QCTL-001 <- QCTL-002")));
    assert!(found.iter().any(|e| e.contains("QCTL-002 <- QCTL-099")));
}

#[test]
fn rejects_archive_not_newest_first() {
    let found = errors(
        "\
schema_version: 1
prefix: QCTL
active: null
queue: []
archive:
  - id: QCTL-001
    title: old
    scope: s
    completed: '2026-01-01'
    outcome: o
    evidence: [e]
  - id: QCTL-002
    title: new
    scope: s
    completed: '2026-08-16'
    outcome: o
    evidence: [e]
",
    );
    assert!(found.iter().any(|e| e.contains("newest-first")));
}

#[test]
fn rejects_missing_plan_path() {
    let found = errors(
        "\
schema_version: 1
prefix: QCTL
active: null
queue:
  - id: QCTL-001
    title: t
    scope: s
    outcome: o
    blocked_by: []
    acceptance: [a]
    plan: missing-plan.md
archive: []
",
    );
    assert!(found.iter().any(|e| e.contains("missing plan")));
}

#[test]
fn next_id_skips_horizon_and_archive() {
    let root = TempDir::new().expect("tempdir");
    let path = root.path().join("tasks.yaml");
    fs::write(
        &path,
        "\
schema_version: 1
prefix: QCTL
active: null
queue: []
archive:
  - id: QCTL-002
    title: t
    scope: s
    completed: '2026-08-16'
    outcome: o
    evidence: [e]
horizon:
  - id: QCTL-010
    title: t
    scope: s
    outcome: o
    kind: deferred
    open: later
",
    )
    .expect("write");
    let ledger = load(&path).expect("parse");
    assert_eq!(qctl::ledger::next_id(&ledger).expect("id"), "QCTL-011");
}
