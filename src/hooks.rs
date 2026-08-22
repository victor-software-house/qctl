//! Git / Lefthook install for `close-from-git`.

use crate::cli::HookInstallArgs;
use crate::ledger::resolve_path;
use anyhow::{Context, Result, ensure};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const PRE_PUSH: &str = "#!/bin/sh\nexec qctl close-from-git --pre-push\n";

const LEFTHOOK_COMMAND: &str = "    qctl-close:\n      run: mise run q close-from-git\n";

pub fn install(args: &HookInstallArgs) -> Result<()> {
    let ledger = resolve_path(&args.ledger);
    let root = git_toplevel(ledger.parent().unwrap_or(&ledger))?;
    if let Some(lefthook) = lefthook_path(&root) {
        return install_lefthook(&lefthook, args.force);
    }
    install_git_hook(&root, args.force)
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

fn install_lefthook(path: &Path, force: bool) -> Result<()> {
    let mut source = fs::read_to_string(path).with_context(|| path.display().to_string())?;
    if source.contains("close-from-git") {
        println!("lefthook already runs close-from-git ({})", path.display());
        return Ok(());
    }
    if !force && source.contains("qctl-close:") {
        anyhow::bail!("{} already has qctl-close (pass --force)", path.display());
    }
    source = insert_lefthook_command(&source);
    fs::write(path, source).with_context(|| path.display().to_string())?;
    println!("wrote {} (mise run q close-from-git)", path.display());
    Ok(())
}

fn insert_lefthook_command(source: &str) -> String {
    if let Some(at) = source.find("\npre-push:\n") {
        let rest = &source[at + "\npre-push:\n".len()..];
        if let Some(commands) = rest.find("\n  commands:\n") {
            let insert_at = at + "\npre-push:\n".len() + commands + "\n  commands:\n".len();
            let mut out = String::new();
            out.push_str(&source[..insert_at]);
            out.push_str(LEFTHOOK_COMMAND);
            out.push_str(&source[insert_at..]);
            return out;
        }
    }
    let mut out = source.to_owned();
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("pre-push:\n  commands:\n");
    out.push_str(LEFTHOOK_COMMAND);
    out
}

fn install_git_hook(root: &Path, force: bool) -> Result<()> {
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
    fs::write(&hook, PRE_PUSH).with_context(|| hook.display().to_string())?;
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
    use super::insert_lefthook_command;

    #[test]
    fn inserts_under_existing_pre_push_commands() {
        let source = "pre-push:\n  skip:\n    - run: test -n \"$CI\"\n  commands:\n    verify:\n      run: mise run verify\n";
        let out = insert_lefthook_command(source);
        assert!(out.contains("qctl-close:"));
        assert!(out.contains("mise run q close-from-git"));
        assert!(out.contains("verify:"));
    }

    #[test]
    fn appends_pre_push_when_missing() {
        let out = insert_lefthook_command("pre-commit:\n  commands:\n    lint:\n      run: true\n");
        assert!(out.contains("pre-push:"));
        assert!(out.contains("mise run q close-from-git"));
    }
}
