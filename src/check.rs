use crate::cli::LedgerArgs;
use crate::ledger::{graph_errors, load, load_value, resolve_path, schema_value};
use anyhow::{Result, bail};
use jsonschema::Validator;

pub fn run(args: &LedgerArgs) -> Result<()> {
    let path = resolve_path(args);
    let schema = schema_value()?;
    let instance = load_value(&path)?;
    let validator = Validator::new(&schema)?;
    let mut errors: Vec<String> = validator
        .iter_errors(&instance)
        .map(|error| format!("{}: {error}", error.instance_path()))
        .collect();

    let ledger = load(&path)?;
    errors.extend(graph_errors(&ledger, &path));

    if errors.is_empty() {
        println!("ok  {}", path.display());
        return Ok(());
    }
    for error in &errors {
        eprintln!("qctl: {error}");
    }
    bail!("{} problem(s) in {}", errors.len(), path.display());
}
