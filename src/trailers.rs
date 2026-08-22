use anyhow::{Context, Result, bail, ensure};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// IDs closed by commit trailers (`Closes CTC-001` / `Completes: PREFIX-NNN`).
///
/// A git failure is an error. `check` reports it; it does not skip. An
/// unborn HEAD is an empty scan: there are no commits, so no trailers.
pub fn closed_ids(root: &Path) -> Result<Vec<(String, String)>> {
    if !head_exists(root)? {
        return Ok(Vec::new());
    }
    Ok(parse_log(&git_log(root, &[])?))
}

/// Outcome of `git rev-parse --show-toplevel`.
pub enum GitRoot {
    /// `start` is inside this repository.
    Root(PathBuf),
    /// `start` is not inside a git repository.
    Absent,
    /// git could not be run, or failed for a reason other than "not a repo".
    Failed(anyhow::Error),
}

/// `git rev-parse --show-toplevel` from `start`, classified by exit code.
///
/// git uses 128 for every fatal error. Absent is 128 plus the C-locale
/// "not a git repository" text (`git_in` sets `LC_ALL=C`).
#[must_use]
pub fn git_root_status(start: &Path) -> GitRoot {
    match git_in(start, ["rev-parse", "--show-toplevel"]) {
        Err(error) => GitRoot::Failed(error),
        Ok(output) if output.status.success() => match String::from_utf8(output.stdout) {
            Ok(path) => GitRoot::Root(PathBuf::from(path.trim())),
            Err(error) => GitRoot::Failed(error.into()),
        },
        Ok(output) if output.status.code() == Some(128) && not_a_work_tree(&output.stderr) => {
            GitRoot::Absent
        }
        Ok(output) => GitRoot::Failed(anyhow::anyhow!(
            "git rev-parse --show-toplevel failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )),
    }
}

/// `git rev-parse --show-toplevel` from `start`.
pub fn git_root(start: &Path) -> Result<PathBuf> {
    match git_root_status(start) {
        GitRoot::Root(root) => Ok(root),
        GitRoot::Absent => bail!("not a git repository"),
        GitRoot::Failed(error) => Err(error),
    }
}

fn head_exists(root: &Path) -> Result<bool> {
    let output = git_in(root, ["rev-parse", "--verify", "--quiet", "HEAD"])?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => bail!(
            "git rev-parse --verify HEAD failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ),
    }
}

fn git_in(root: &Path, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Result<Output> {
    Command::new("git")
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .context("git")
}

fn git_log(root: &Path, rev: &[&str]) -> Result<String> {
    ensure!(
        rev.iter().all(|part| !part.starts_with('-')),
        "range is a revision, not a git option"
    );
    let mut args: Vec<&str> = vec!["log", "--format=%H%x00%B%x1e"];
    args.extend(rev);
    let output = git_in(root, args)?;
    ensure!(
        output.status.success(),
        "git log failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Same scan, but a git failure is an error. `rev` is passed to `git log`
/// after the format: empty means the whole reachable history, `main..HEAD`
/// means that range, a SHA means that commit and its ancestors.
pub fn closed_ids_rev(root: &Path, rev: &[&str]) -> Result<Vec<(String, String)>> {
    Ok(parse_log(&git_log(root, rev)?))
}

/// Pre-push stdin: one `<local-ref> <local-sha> <remote-ref> <remote-sha>`
/// line per ref. A zero remote SHA is a new ref (log the local SHA).
pub fn closed_ids_pre_push(root: &Path, stdin: &str) -> Result<Vec<(String, String)>> {
    let mut found = Vec::new();
    for line in stdin.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        ensure!(
            parts.len() == 4,
            "pre-push line is not <local-ref> <local-sha> <remote-ref> <remote-sha>"
        );
        let local_sha = parts[1];
        let remote_sha = parts[3];
        if is_zero_sha(local_sha) {
            continue;
        }
        let batch = if is_zero_sha(remote_sha) {
            closed_ids_rev(root, &[local_sha])?
        } else {
            let range = format!("{remote_sha}..{local_sha}");
            closed_ids_rev(root, &[range.as_str()])?
        };
        found.extend(batch);
    }
    Ok(found)
}

fn is_zero_sha(sha: &str) -> bool {
    !sha.is_empty() && sha.chars().all(|ch| ch == '0')
}

#[must_use]
pub fn parse_log(log: &str) -> Vec<(String, String)> {
    let mut found = Vec::new();
    for record in log.split('\u{1e}') {
        let record = record.trim();
        if record.is_empty() {
            continue;
        }
        let Some((sha, body)) = record.split_once('\0') else {
            continue;
        };
        for id in parse_body(body) {
            found.push((id, sha.to_owned()));
        }
    }
    found
}

fn parse_body(body: &str) -> Vec<String> {
    let mut ids = Vec::new();
    for line in body.lines() {
        let line = line.trim();
        if let Some(id) = line.strip_prefix("Closes ") {
            push_id(&mut ids, id);
        } else if let Some(id) = line.strip_prefix("Completes: ") {
            push_id(&mut ids, id);
        }
    }
    ids
}

fn push_id(ids: &mut Vec<String>, raw: &str) {
    let id = raw.trim();
    if id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') && id.contains('-') {
        ids.push(id.to_owned());
    }
}

fn not_a_work_tree(stderr: &[u8]) -> bool {
    String::from_utf8_lossy(stderr)
        .to_ascii_lowercase()
        .lines()
        .any(|line| {
            let message = line.trim().strip_prefix("fatal: ").unwrap_or(line.trim());
            message.starts_with("not a git repository")
                || message.contains("must be run in a work tree")
        })
}

#[cfg(test)]
mod tests {
    use super::parse_log;
    use indoc::indoc;

    #[test]
    fn reads_closes_and_completes() {
        let log = indoc! {"
            abc\0feat: chassis

            Closes CTC-001
            \u{1e}def\0chore: note

            Completes: CTC-002
        "};
        let ids = parse_log(log);
        assert_eq!(
            ids,
            vec![
                ("CTC-001".into(), "abc".into()),
                ("CTC-002".into(), "def".into()),
            ]
        );
    }

    #[test]
    fn ignores_subject_slogans() {
        let log = indoc! {"
            abc\0Closes CTC-001 in the subject only

            No trailer.
        "};
        assert_eq!(parse_log(log), Vec::new());
    }

    #[test]
    fn pre_push_skips_a_deleted_ref() {
        let dir = tempfile::TempDir::new().unwrap();
        let zeros = "0".repeat(40);
        let stdin = format!("refs/heads/gone {zeros} refs/heads/gone abcdef\n");
        let found = super::closed_ids_pre_push(dir.path(), &stdin).unwrap();
        assert_eq!(found, Vec::new());
    }

    #[test]
    fn range_cannot_be_a_git_option() {
        let dir = tempfile::TempDir::new().unwrap();
        let err = super::closed_ids_rev(dir.path(), &["--output=/tmp/qctl-git"]).unwrap_err();
        assert!(format!("{err:#}").contains("revision"), "{err:#}");
    }

    #[test]
    fn git_root_status_absent_vs_missing_dir() {
        let dir = tempfile::TempDir::new().unwrap();
        assert!(matches!(
            super::git_root_status(dir.path()),
            super::GitRoot::Absent
        ));
        let missing = dir.path().join("nope");
        assert!(matches!(
            super::git_root_status(&missing),
            super::GitRoot::Failed(_)
        ));
    }

    #[test]
    fn not_a_work_tree_is_case_insensitive() {
        assert!(super::not_a_work_tree(
            b"fatal: Not a git repository (or any of the parent directories): .git"
        ));
        assert!(super::not_a_work_tree(
            b"fatal: this operation must be run in a work tree"
        ));
        assert!(super::not_a_work_tree(
            b"fatal: git rev-parse: this operation must be run in a work tree"
        ));
        assert!(!super::not_a_work_tree(b"fatal: cannot change to '/nope'"));
        assert!(!super::not_a_work_tree(
            b"fatal: cannot change to '/tmp/not a git repository': No such file or directory"
        ));
    }
}
