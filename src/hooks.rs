//! Git hooks that call `qctl close-from-git`.

use crate::cli::HookInstallArgs;
use crate::ledger::resolve_path;
use anyhow::{Context, Result, ensure};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const PRE_PUSH: &str = "#!/bin/sh\nexec qctl close-from-git --pre-push\n";

pub fn install(args: &HookInstallArgs) -> Result<()> {
    let ledger = resolve_path(&args.ledger);
    let root = ledger.parent().unwrap_or(&ledger);
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
    if hook.exists() && !args.force {
        bail_exists(&hook)?;
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

fn bail_exists(hook: &std::path::Path) -> Result<()> {
    anyhow::bail!("{} already exists (pass --force)", hook.display())
}
