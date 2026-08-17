use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// IDs closed by commit trailers (`Closes CTC-001` / `Completes: CTC-001`).
pub fn closed_ids(root: &Path) -> Result<Vec<(String, String)>> {
    if !root.join(".git").exists() && find_git_dir(root).is_none() {
        return Ok(Vec::new());
    }
    let output = Command::new("git")
        .args([
            "-C",
            &root.display().to_string(),
            "log",
            "--format=%H%x00%B%x1e",
        ])
        .output()
        .context("git log")?;
    if !output.status.success() {
        return Ok(Vec::new());
    }
    Ok(parse_log(&String::from_utf8_lossy(&output.stdout)))
}

fn find_git_dir(root: &Path) -> Option<std::path::PathBuf> {
    let output = Command::new("git")
        .args(["-C", &root.display().to_string(), "rev-parse", "--git-dir"])
        .output()
        .ok()?;
    output.status.success().then_some(root.to_path_buf())
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
        assert!(parse_log(log).is_empty());
    }
}
