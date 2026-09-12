use anyhow::{Result, bail};
use std::collections::HashMap;

pub(crate) struct Row<'a> {
    pub id: &'a str,
    pub blockers: &'a [String],
}

/// Require every blocker to be queued before the row that names it.
pub(crate) fn validate(rows: &[Row<'_>]) -> Result<()> {
    let positions: HashMap<&str, usize> = rows
        .iter()
        .enumerate()
        .map(|(index, row)| (row.id, index))
        .collect();
    for (index, row) in rows.iter().enumerate() {
        for blocker in row.blockers {
            match positions.get(blocker.as_str()) {
                Some(at) if *at < index => {}
                Some(_) => bail!("{} <- {blocker} is not earlier", row.id),
                None => bail!("{} <- {blocker} is not queued", row.id),
            }
        }
    }
    Ok(())
}
