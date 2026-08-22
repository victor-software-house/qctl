use crate::cli::CheckArgs;
use crate::ledger::{graph_errors, load_value, read, resolve_path, schema_value};
use crate::trailers;
use anyhow::{Result, bail};
use std::collections::HashSet;

pub fn run(args: &CheckArgs) -> Result<()> {
    let path = resolve_path(&args.ledger);
    let schema = schema_value()?;
    let instance = load_value(&path)?;
    // `format` is an annotation by default, which would let `completed: last
    // Tuesday` through the one keyword that describes it.
    let validator = jsonschema::options()
        .should_validate_formats(true)
        .build(&schema)?;
    let mut errors: Vec<String> = validator
        .iter_errors(&instance)
        .map(|error| format!("{}: {error}", error.instance_path()))
        .collect();

    // A wrong value must not hide a wrong graph, so the rows are read without
    // being judged and every rule reports in the same pass. The values are the
    // schema's business here — garde states the same rules for the verbs, and
    // repeating it would print each defect twice.
    match read(&path) {
        Ok(ledger) => {
            errors.extend(graph_errors(&ledger, &path));
            if !args.no_git {
                errors.extend(trailer_errors(&ledger, &path));
            }
        }
        Err(error) => errors.push(format!("{error:#}")),
    }

    if errors.is_empty() {
        println!("ok  {}", path.display());
        return Ok(());
    }
    for error in &errors {
        eprintln!("qctl: {error}");
    }
    bail!(
        "{} problem(s) in {}: {}",
        errors.len(),
        path.display(),
        errors.join("; ")
    );
}

fn trailer_errors(ledger: &crate::ledger::Ledger, path: &std::path::Path) -> Vec<String> {
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => return vec![format!("git trailer scan failed: {error}")],
    };
    let ledger_dir = path.parent().unwrap_or(path);
    let skip = "git trailer scan skipped: ledger is not in the current repository; pass --no-git";
    let ledger_root = match trailers::git_root_status(ledger_dir) {
        trailers::GitRoot::Failed(error) => {
            return vec![format!("git trailer scan failed: {error:#}")];
        }
        trailers::GitRoot::Absent => return vec![skip.into()],
        trailers::GitRoot::Root(root) => root,
    };
    let cwd_root = match trailers::git_root_status(&cwd) {
        trailers::GitRoot::Failed(error) => {
            return vec![format!("git trailer scan failed: {error:#}")];
        }
        trailers::GitRoot::Absent => return vec![skip.into()],
        trailers::GitRoot::Root(root) => root,
    };
    let ledger_root = ledger_root.canonicalize().unwrap_or(ledger_root);
    let cwd_root = cwd_root.canonicalize().unwrap_or(cwd_root);
    if ledger_root != cwd_root {
        return vec![skip.into()];
    }
    let closed = match trailers::closed_ids(&ledger_root) {
        Ok(closed) => closed,
        Err(error) => return vec![format!("git trailer scan failed: {error:#}")],
    };
    let queued: HashSet<_> = ledger.queue.iter().map(|task| task.id.as_str()).collect();
    let mut errors = Vec::new();
    let mut seen = HashSet::new();
    for (id, sha) in closed {
        if !queued.contains(id.as_str()) || !seen.insert(id.clone()) {
            continue;
        }
        errors.push(format!(
            "{id} is closed by {sha} but still queued; archive it (Closes/Completes trailer)"
        ));
    }
    errors
}
