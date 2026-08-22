//! `close-from-git` and `hook install`.

mod common;

use common::{qctl, stderr, stdout};
use indoc::indoc;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use tempfile::TempDir;

fn git(root: &std::path::Path, args: &[&str]) {
    let status = Command::new("git")
        .args(["-C", &root.display().to_string()])
        .args(args)
        .status()
        .expect("git");
    assert!(status.success(), "git {args:?}");
}

fn repo() -> TempDir {
    let root = TempDir::new().expect("tmp");
    git(root.path(), &["init"]);
    git(root.path(), &["config", "user.email", "t@example.com"]);
    git(root.path(), &["config", "user.name", "t"]);
    fs::write(
        root.path().join("tasks.yaml"),
        indoc! {"
            schema_version: 3
            prefix: CTC
            active: CTC-001
            queue:
              - id: CTC-001
                title: Chassis
                scope: ctl-core
                outcome: Crate exists.
                blocked_by: []
                acceptance:
                  - prelude ships
            archive: []
        "},
    )
    .expect("ledger");
    root
}

#[test]
fn close_from_git_archives_a_queued_id() {
    let root = repo();
    fs::write(root.path().join("note"), "x").expect("note");
    git(root.path(), &["add", "tasks.yaml", "note"]);
    git(
        root.path(),
        &["commit", "-m", "feat: chassis", "-m", "Closes CTC-001"],
    );
    let path = root.path().join("tasks.yaml");
    let path = path.to_str().expect("utf-8");
    let output = qctl(&["close-from-git", "-f", path]);
    assert!(output.status.success(), "{}", stderr(&output));
    let body = fs::read_to_string(path).expect("read");
    assert!(body.contains("archive:"), "{body}");
    assert!(body.contains("CTC-001"), "{body}");
    assert!(
        body.contains("active: null") || body.contains("active: null\n"),
        "{body}"
    );
    assert!(!body.contains("queue:\n  - id: CTC-001"), "{body}");
    let sha = String::from_utf8(
        Command::new("git")
            .args([
                "-C",
                &root.path().display().to_string(),
                "rev-parse",
                "HEAD",
            ])
            .output()
            .expect("sha")
            .stdout,
    )
    .expect("utf-8");
    assert!(body.contains(sha.trim()), "{body}");
    let check = qctl(&["check", "-f", path]);
    assert!(check.status.success(), "{}", stderr(&check));
}

#[test]
fn close_from_git_ignores_a_subject_slogan() {
    let root = repo();
    fs::write(root.path().join("note"), "x").expect("note");
    git(root.path(), &["add", "tasks.yaml", "note"]);
    git(
        root.path(),
        &["commit", "-m", "Closes CTC-001 in the subject only"],
    );
    let path = root.path().join("tasks.yaml");
    let path = path.to_str().expect("utf-8");
    let output = qctl(&["close-from-git", "-f", path]);
    assert!(output.status.success(), "{}", stderr(&output));
    let body = fs::read_to_string(path).expect("read");
    assert!(body.contains("active: CTC-001"), "{body}");
}

#[test]
fn close_from_git_pre_push_refuses_to_amend() {
    let root = repo();
    fs::write(root.path().join("note"), "x").expect("note");
    git(root.path(), &["add", "tasks.yaml", "note"]);
    git(
        root.path(),
        &["commit", "-m", "feat: chassis", "-m", "Closes CTC-001"],
    );
    let sha = String::from_utf8(
        Command::new("git")
            .args([
                "-C",
                &root.path().display().to_string(),
                "rev-parse",
                "HEAD",
            ])
            .output()
            .expect("sha")
            .stdout,
    )
    .expect("utf-8");
    let sha = sha.trim();
    let zeros = "0".repeat(40);
    let stdin = format!("refs/heads/main {sha} refs/heads/main {zeros}\n");
    let path = root.path().join("tasks.yaml");
    let output = Command::new(env!("CARGO_BIN_EXE_qctl"))
        .args([
            "close-from-git",
            "--pre-push",
            "-f",
            path.to_str().expect("utf-8"),
        ])
        .env_remove("TASKS_LEDGER")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            child
                .stdin
                .as_mut()
                .expect("stdin")
                .write_all(stdin.as_bytes())?;
            child.wait_with_output()
        })
        .expect("run");
    assert!(!output.status.success(), "hook must not amend");
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(err.contains("does not amend"), "{err}");
    assert!(err.contains("CTC-001"), "{err}");
}

#[test]
fn hook_install_writes_pre_push() {
    let root = repo();
    git(root.path(), &["add", "tasks.yaml"]);
    git(root.path(), &["commit", "-m", "init"]);
    let path = root.path().join("tasks.yaml");
    let output = qctl(&["hook", "install", "-f", path.to_str().expect("utf-8")]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains("pre-push"), "{}", stdout(&output));
    let hook = String::from_utf8(
        Command::new("git")
            .args([
                "-C",
                &root.path().display().to_string(),
                "rev-parse",
                "--git-path",
                "hooks/pre-push",
            ])
            .output()
            .expect("path")
            .stdout,
    )
    .expect("utf-8");
    let hook = root.path().join(hook.trim());
    let body = fs::read_to_string(&hook).expect("hook");
    assert!(body.contains("close-from-git --pre-push"), "{body}");
    let mode = fs::metadata(&hook).expect("meta").permissions().mode();
    assert_eq!(mode & 0o111, 0o111, "hook is not executable: {mode:o}");
    let again = qctl(&["hook", "install", "-f", path.to_str().expect("utf-8")]);
    assert!(!again.status.success());
    assert!(
        stderr(&again).contains("already exists"),
        "{}",
        stderr(&again)
    );
}
