//! Git / Lefthook install for `close-from-git`.
//!
//! Lefthook has no API to add a command, no `extends`, and no merge of
//! foreign config. This crate does not edit `lefthook.yml`. When that file
//! exists, install prints the `mise run q` snippet to add. Otherwise it
//! writes a git `pre-push`.

use crate::cli::HookInstallArgs;
use crate::ledger::resolve_path;
use anyhow::{Context, Result, ensure};
use indoc::formatdoc;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn install(args: &HookInstallArgs) -> Result<()> {
    let ledger = resolve_path(&args.ledger);
    let ledger = canonicalize_ledger(&ledger)?;
    let root = git_toplevel(ledger.parent().unwrap_or(&ledger))?;
    let root = root
        .canonicalize()
        .with_context(|| root.display().to_string())?;
    let rel = ledger_rel(&ledger, &root)?;
    if let Some(lefthook) = lefthook_path(&root) {
        return install_lefthook(&lefthook, &rel, args.force);
    }
    install_git_hook(&root, &rel, args.force)
}

fn canonicalize_ledger(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return path
            .canonicalize()
            .with_context(|| path.display().to_string());
    }
    let name = path
        .file_name()
        .with_context(|| format!("{} has no file name", path.display()))?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    Ok(parent
        .canonicalize()
        .with_context(|| parent.display().to_string())?
        .join(name))
}

fn git_toplevel(start: &Path) -> Result<PathBuf> {
    let output = Command::new("git")
        .args([
            "-C",
            &start.display().to_string(),
            "rev-parse",
            "--show-toplevel",
        ])
        .output()
        .context("git rev-parse --show-toplevel")?;
    ensure!(
        output.status.success(),
        "git rev-parse --show-toplevel failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    let path = String::from_utf8(output.stdout).context("toplevel is not utf-8")?;
    Ok(PathBuf::from(path.trim()))
}

fn lefthook_path(root: &Path) -> Option<PathBuf> {
    for name in ["lefthook.yml", "lefthook.yaml"] {
        let path = root.join(name);
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

fn ledger_rel(ledger: &Path, root: &Path) -> Result<String> {
    let rel = ledger
        .strip_prefix(root)
        .with_context(|| format!("{} is not inside {}", ledger.display(), root.display()))?;
    let rel = rel.to_string_lossy();
    ensure!(
        !rel.contains('\''),
        "ledger path must not contain a single quote"
    );
    Ok(rel.replace('\\', "/"))
}

fn lefthook_snippet(rel: &str) -> String {
    formatdoc! {"
        # add under pre-push.commands in lefthook.yml
        {pad}qctl-close:
        {pad}  run: mise run q close-from-git -f '{rel}'
    ", pad = "    "}
}

fn git_hook_body(rel: &str) -> String {
    formatdoc! {"
        #!/bin/sh
        exec qctl close-from-git --pre-push -f '{rel}'
    "}
}

fn install_lefthook(path: &Path, rel: &str, force: bool) -> Result<()> {
    let source = fs::read_to_string(path).with_context(|| path.display().to_string())?;
    if source.contains("close-from-git") {
        println!("lefthook already runs close-from-git ({})", path.display());
        return Ok(());
    }
    print!("{}", lefthook_snippet(rel));
    eprintln!(
        "qctl: add that under pre-push.commands in {} (qctl does not edit Lefthook config)",
        path.display()
    );
    ensure!(
        !force,
        "--force does not apply when {path} exists; qctl does not edit Lefthook config",
        path = path.display()
    );
    anyhow::bail!("{path} present; hook not installed", path = path.display())
}

fn install_git_hook(root: &Path, rel: &str, force: bool) -> Result<()> {
    let output = Command::new("git")
        .args([
            "-C",
            &root.display().to_string(),
            "rev-parse",
            "--git-path",
            "hooks/pre-push",
        ])
        .output()
        .context("git rev-parse --git-path hooks/pre-push")?;
    ensure!(
        output.status.success(),
        "git rev-parse --git-path hooks/pre-push failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    let reported = String::from_utf8(output.stdout).context("hook path is not utf-8")?;
    let reported = reported.trim();
    let hook = {
        let path = PathBuf::from(reported);
        if path.is_absolute() {
            path
        } else {
            root.join(path)
        }
    };
    if hook.exists() && !force {
        anyhow::bail!("{} already exists (pass --force)", hook.display());
    }
    if let Some(parent) = hook.parent() {
        fs::create_dir_all(parent).with_context(|| parent.display().to_string())?;
    }
    fs::write(&hook, git_hook_body(rel)).with_context(|| hook.display().to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&hook)
            .with_context(|| hook.display().to_string())?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&hook, permissions).with_context(|| hook.display().to_string())?;
    }
    println!("wrote {}", hook.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{canonicalize_ledger, git_hook_body, ledger_rel, lefthook_snippet};
    use indoc::indoc;
    use std::path::Path;

    #[test]
    fn lefthook_snippet_is_the_command_to_paste() {
        assert_eq!(
            lefthook_snippet("tasks.yaml"),
            indoc! {"
                # add under pre-push.commands in lefthook.yml
                    qctl-close:
                      run: mise run q close-from-git -f 'tasks.yaml'
            "}
        );
    }

    #[test]
    fn lefthook_snippet_quotes_a_nested_path() {
        assert_eq!(
            lefthook_snippet("queue/tasks.yaml"),
            indoc! {"
                # add under pre-push.commands in lefthook.yml
                    qctl-close:
                      run: mise run q close-from-git -f 'queue/tasks.yaml'
            "}
        );
    }

    #[test]
    fn git_hook_body_is_the_script() {
        assert_eq!(
            git_hook_body("tasks.yaml"),
            indoc! {"
                #!/bin/sh
                exec qctl close-from-git --pre-push -f 'tasks.yaml'
            "}
        );
    }

    #[test]
    fn git_hook_body_quotes_a_nested_path() {
        assert_eq!(
            git_hook_body("queue/tasks.yaml"),
            indoc! {"
                #!/bin/sh
                exec qctl close-from-git --pre-push -f 'queue/tasks.yaml'
            "}
        );
    }

    #[test]
    fn canonicalize_ledger_joins_a_missing_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let missing = dir.path().join("tasks.yaml");
        assert!(!missing.exists());
        let got = canonicalize_ledger(&missing).unwrap();
        assert_eq!(got, dir.path().canonicalize().unwrap().join("tasks.yaml"));
    }

    #[test]
    fn ledger_rel_refuses_a_single_quote() {
        let err = ledger_rel(Path::new("/repo/it's.yaml"), Path::new("/repo")).unwrap_err();
        assert!(format!("{err:#}").contains("single quote"), "{err:#}");
    }
}
