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

fn git_hook(root: &std::path::Path) -> std::path::PathBuf {
    let hook = String::from_utf8(
        Command::new("git")
            .args([
                "-C",
                &root.display().to_string(),
                "rev-parse",
                "--git-path",
                "hooks/pre-push",
            ])
            .output()
            .expect("path")
            .stdout,
    )
    .expect("utf-8");
    let hook = hook.trim();
    let path = std::path::PathBuf::from(hook);
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}

fn sha(root: &std::path::Path) -> String {
    let out = Command::new("git")
        .args(["-C", &root.display().to_string(), "rev-parse", "HEAD"])
        .output()
        .expect("sha");
    String::from_utf8(out.stdout)
        .expect("utf-8")
        .trim()
        .to_owned()
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
    assert!(body.contains("active: null"), "{body}");
    assert!(
        !body.contains(indoc! {"
            queue:
              - id: CTC-001
        "}),
        "{body}"
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
    let hook = git_hook(root.path());
    assert_eq!(
        fs::read_to_string(&hook).expect("hook"),
        indoc! {"
            #!/bin/sh
            exec qctl close-from-git --pre-push -f 'tasks.yaml'
        "}
    );
    let mode = fs::metadata(&hook).expect("meta").permissions().mode();
    assert_eq!(mode & 0o111, 0o111, "hook is not executable: {mode:o}");
    let again = qctl(&["hook", "install", "-f", path.to_str().expect("utf-8")]);
    assert!(!again.status.success());
    assert!(
        stderr(&again).contains("already exists"),
        "{}",
        stderr(&again)
    );
    let forced = qctl(&[
        "hook",
        "install",
        "--force",
        "-f",
        path.to_str().expect("utf-8"),
    ]);
    assert!(forced.status.success(), "{}", stderr(&forced));
    assert_eq!(
        fs::read_to_string(&hook).expect("hook"),
        indoc! {"
            #!/bin/sh
            exec qctl close-from-git --pre-push -f 'tasks.yaml'
        "}
    );
}

#[test]
fn hook_install_prefers_lefthook_and_mise_q() {
    let root = repo();
    git(root.path(), &["add", "tasks.yaml"]);
    git(root.path(), &["commit", "-m", "init"]);
    fs::write(
        root.path().join("lefthook.yml"),
        indoc! {"
            pre-push:
              commands:
                verify:
                  run: mise run verify
        "},
    )
    .expect("lefthook");
    let path = root.path().join("tasks.yaml");
    let before = fs::read_to_string(root.path().join("lefthook.yml")).expect("before");
    let output = qctl(&["hook", "install", "-f", path.to_str().expect("utf-8")]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        stdout(&output),
        indoc! {"
            # add under pre-push.commands in lefthook.yml
                qctl-close:
                  run: mise run q close-from-git -f 'tasks.yaml'
        "}
    );
    assert!(
        stderr(&output).contains("does not edit Lefthook config"),
        "{}",
        stderr(&output)
    );
    let after = fs::read_to_string(root.path().join("lefthook.yml")).expect("after");
    assert_eq!(after, before);
    assert!(after.contains("verify:"), "{after}");
    assert!(!after.contains("close-from-git"), "{after}");
    assert!(
        !git_hook(root.path()).exists(),
        "must not write a git hook when lefthook.yml exists"
    );
}

#[test]
fn hook_install_reports_when_lefthook_already_runs_close_from_git() {
    let root = repo();
    git(root.path(), &["add", "tasks.yaml"]);
    git(root.path(), &["commit", "-m", "init"]);
    let leftover = indoc! {"
        pre-push:
          commands:
            qctl-close:
              run: mise run q close-from-git -f 'tasks.yaml'
    "};
    fs::write(root.path().join("lefthook.yml"), leftover).expect("lefthook");
    let path = root.path().join("tasks.yaml");
    let output = qctl(&["hook", "install", "-f", path.to_str().expect("utf-8")]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(
        stdout(&output).contains("already runs close-from-git"),
        "{}",
        stdout(&output)
    );
    assert_eq!(
        fs::read_to_string(root.path().join("lefthook.yml")).expect("after"),
        leftover
    );
}

#[test]
fn hook_install_sees_lefthook_yaml() {
    let root = repo();
    git(root.path(), &["add", "tasks.yaml"]);
    git(root.path(), &["commit", "-m", "init"]);
    fs::write(
        root.path().join("lefthook.yaml"),
        indoc! {"
            pre-push:
              commands:
                verify:
                  run: mise run verify
        "},
    )
    .expect("lefthook");
    let path = root.path().join("tasks.yaml");
    let before = fs::read_to_string(root.path().join("lefthook.yaml")).expect("before");
    let output = qctl(&["hook", "install", "-f", path.to_str().expect("utf-8")]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(
        stdout(&output).contains("qctl-close:"),
        "{}",
        stdout(&output)
    );
    assert_eq!(
        fs::read_to_string(root.path().join("lefthook.yaml")).expect("after"),
        before
    );
    assert!(!git_hook(root.path()).exists());
}

#[test]
fn hook_install_works_when_the_ledger_is_missing() {
    let root = repo();
    git(root.path(), &["add", "tasks.yaml"]);
    git(root.path(), &["commit", "-m", "init"]);
    let missing = root.path().join("queue").join("tasks.yaml");
    fs::create_dir_all(missing.parent().expect("parent")).expect("dir");
    let output = qctl(&["hook", "install", "-f", missing.to_str().expect("utf-8")]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        fs::read_to_string(git_hook(root.path())).expect("hook"),
        indoc! {"
            #!/bin/sh
            exec qctl close-from-git --pre-push -f 'queue/tasks.yaml'
        "}
    );
}

#[test]
fn close_from_git_archives_a_completes_trailer() {
    let root = repo();
    fs::write(root.path().join("note"), "x").expect("note");
    git(root.path(), &["add", "tasks.yaml", "note"]);
    git(
        root.path(),
        &["commit", "-m", "feat: chassis", "-m", "Completes: CTC-001"],
    );
    let path = root.path().join("tasks.yaml");
    let output = qctl(&["close-from-git", "-f", path.to_str().expect("utf-8")]);
    assert!(output.status.success(), "{}", stderr(&output));
    let body = fs::read_to_string(&path).expect("read");
    assert!(body.contains("active: null"), "{body}");
    assert!(body.contains(sha(root.path()).as_str()), "{body}");
}
