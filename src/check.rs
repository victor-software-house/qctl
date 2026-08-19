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
    let root = path.parent().unwrap_or(path);
    let Ok(closed) = trailers::closed_ids(root) else {
        return Vec::new();
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
