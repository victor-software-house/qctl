use anyhow::{Context, Result, ensure};
use std::path::Path;
use std::process::Command;

/// IDs closed by commit trailers (`Closes CTC-001` / `Completes: CTC-001`).
///
/// A missing git repo, or a `git log` that cannot run, is an empty list —
/// `check` skips rather than failing the whole run (QCTL-026).
pub fn closed_ids(root: &Path) -> Result<Vec<(String, String)>> {
    match git_log(root, &[]) {
        Ok(text) => Ok(parse_log(&text)),
        Err(_) => Ok(Vec::new()),
    }
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
        let batch = if remote_sha.chars().all(|ch| ch == '0') {
            closed_ids_rev(root, &[local_sha])?
        } else {
            let range = format!("{remote_sha}..{local_sha}");
            closed_ids_rev(root, &[range.as_str()])?
        };
        found.extend(batch);
    }
    Ok(found)
}

fn git_log(root: &Path, rev: &[&str]) -> Result<String> {
    let mut command = Command::new("git");
    command.args([
        "-C",
        &root.display().to_string(),
        "log",
        "--format=%H%x00%B%x1e",
    ]);
    command.args(rev);
    let output = command.output().context("git log")?;
    ensure!(
        output.status.success(),
        "git log failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
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
        let log = "abc\0Closes CTC-001 in the subject only\n\nNo trailer.\n";
        assert_eq!(parse_log(log), Vec::new());
    }
}
