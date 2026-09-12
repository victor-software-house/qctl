use crate::cli::{EditArgs, EditPosition};
use crate::ledger::Ledger;
use crate::ledger::order::{self, Row};
use anyhow::{Context, Result, bail};

pub(super) fn destination(
    ledger: &Ledger,
    args: &EditArgs,
    edited_blockers: Option<&[String]>,
) -> Result<usize> {
    let edited_blockers = edited_blockers.context("queued row has no blocker state")?;
    let ids: Vec<&str> = ledger.queue.iter().map(|row| row.id.as_str()).collect();
    let current = ids
        .iter()
        .position(|queued| *queued == args.id)
        .expect("located");
    let destination = index(&ids, current, args.position.as_ref())?;

    if destination == 0
        && let Some(active) = ledger.active.as_deref()
        && active != args.id
    {
        bail!(
            "edit would make {} queue[0] while active is {active}",
            args.id
        );
    }
    if ledger.active.as_deref() == Some(args.id.as_str()) && destination != 0 {
        bail!("edit would leave active {} off queue[0]", args.id);
    }

    let mut ordered: Vec<Row<'_>> = ledger
        .queue
        .iter()
        .filter(|row| row.id != args.id)
        .map(|row| Row {
            id: &row.id,
            blockers: &row.blocked_by,
        })
        .collect();
    ordered.insert(
        destination,
        Row {
            id: &args.id,
            blockers: edited_blockers,
        },
    );
    order::validate(&ordered)?;
    Ok(destination)
}

fn index(ids: &[&str], current: usize, position: Option<&EditPosition>) -> Result<usize> {
    let Some(position) = position else {
        return Ok(current);
    };
    let remaining: Vec<&str> = ids
        .iter()
        .copied()
        .filter(|queued| *queued != ids[current])
        .collect();
    match position {
        EditPosition::Front => Ok(0),
        EditPosition::Back => Ok(remaining.len()),
        EditPosition::Before(before) => remaining
            .iter()
            .position(|queued| *queued == before)
            .with_context(|| format!("{before} is not queued")),
        EditPosition::After(after) => {
            let index = remaining
                .iter()
                .position(|queued| *queued == after)
                .with_context(|| format!("{after} is not queued"))?;
            Ok(index + 1)
        }
    }
}
