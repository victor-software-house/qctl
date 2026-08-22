//! Git / Lefthook install for `close-from-git`.

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

fn lefthook_command(rel: &str) -> String {
    formatdoc! {"
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
    let mut source = fs::read_to_string(path).with_context(|| path.display().to_string())?;
    if source.contains("close-from-git") {
        println!("lefthook already runs close-from-git ({})", path.display());
        return Ok(());
    }
    if !force && source.contains("qctl-close:") {
        anyhow::bail!("{} already has qctl-close (pass --force)", path.display());
    }
    source = insert_lefthook_command(&source, rel);
    fs::write(path, source).with_context(|| path.display().to_string())?;
    println!(
        "wrote {} (mise run q close-from-git -f {rel})",
        path.display()
    );
    Ok(())
}

fn insert_lefthook_command(source: &str, rel: &str) -> String {
    let command = lefthook_command(rel);
    if let Some((from, to)) = pre_push_span(source) {
        let block = &source[from..to];
        if let Some(commands) = block.find("\n  commands:\n") {
            let insert_at = from + commands + "\n  commands:\n".len();
            let mut out = String::new();
            out.push_str(&source[..insert_at]);
            out.push_str(&command);
            out.push_str(&source[insert_at..]);
            return out;
        }
        let mut out = String::new();
        out.push_str(&source[..to]);
        if !block.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("  commands:\n");
        out.push_str(&command);
        out.push_str(&source[to..]);
        return out;
    }
    let mut out = source.to_owned();
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("pre-push:\n  commands:\n");
    out.push_str(&command);
    out
}

/// Byte range of the `pre-push:` mapping, up to the next top-level key.
fn pre_push_span(source: &str) -> Option<(usize, usize)> {
    let body = if source.starts_with("pre-push:\n") {
        "pre-push:\n".len()
    } else {
        let at = source.find("\npre-push:\n")?;
        at + 1 + "pre-push:\n".len()
    };
    let rest = &source[body..];
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if offset > 0 && line.starts_with(|ch: char| ch.is_ascii_alphabetic()) {
            return Some((body.saturating_sub("pre-push:\n".len()), body + offset));
        }
        offset += line.len();
    }
    let from = body.saturating_sub("pre-push:\n".len());
    Some((from, source.len()))
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
    use super::insert_lefthook_command;

    #[test]
    fn inserts_under_existing_pre_push_commands() {
        let source = "pre-push:\n  skip:\n    - run: test -n \"$CI\"\n  commands:\n    verify:\n      run: mise run verify\n";
        let out = insert_lefthook_command(source, "tasks.yaml");
        assert!(out.contains("qctl-close:"));
        assert!(out.contains("mise run q close-from-git -f 'tasks.yaml'"));
        assert!(out.contains("verify:"));
    }

    #[test]
    fn appends_pre_push_when_missing() {
        let out = insert_lefthook_command(
            "pre-commit:\n  commands:\n    lint:\n      run: true\n",
            "queue/tasks.yaml",
        );
        assert!(out.contains("pre-push:"));
        assert!(out.contains("mise run q close-from-git -f 'queue/tasks.yaml'"));
        let lint_at = out.find("lint:").expect("lint");
        let qctl_at = out.find("qctl-close:").expect("qctl");
        assert!(qctl_at > lint_at, "{out}");
    }

    #[test]
    fn does_not_insert_under_pre_commit_commands() {
        let source = "pre-push:\n  skip:\n    - run: test -n \"$CI\"\npre-commit:\n  commands:\n    lint:\n      run: true\n";
        let out = insert_lefthook_command(source, "tasks.yaml");
        let pre_commit = out.find("pre-commit:").expect("pre-commit");
        let qctl = out.find("qctl-close:").expect("qctl");
        assert!(qctl < pre_commit, "{out}");
        assert!(out.contains("lint:"));
    }
}
