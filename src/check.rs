use crate::cli::CheckArgs;
use crate::ledger::{graph_errors, load, load_value, resolve_path, schema_value};
use crate::trailers;
use anyhow::{Result, bail};
use jsonschema::Validator;
use std::collections::HashSet;

pub fn run(args: &CheckArgs) -> Result<()> {
    let path = resolve_path(&args.ledger);
    let schema = schema_value()?;
    let instance = load_value(&path)?;
    let validator = Validator::new(&schema)?;
    let mut errors: Vec<String> = validator
        .iter_errors(&instance)
        .map(|error| format!("{}: {error}", error.instance_path()))
        .collect();

    let ledger = load(&path)?;
    errors.extend(graph_errors(&ledger, &path));
    if !args.no_git {
        errors.extend(trailer_errors(&ledger, &path));
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
