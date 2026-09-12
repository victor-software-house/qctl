use crate::schema::Ledger;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub(super) const MAX_TASK_NUMBER: u32 = 999_999;

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
enum Status {
    Queue,
    Archive,
    Horizon,
}

impl fmt::Display for Status {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Queue => "queue",
            Self::Archive => "archive",
            Self::Horizon => "horizon",
        })
    }
}

struct Task<'a> {
    id: &'a str,
    status: Status,
}

pub(super) fn errors(ledger: &Ledger) -> Vec<String> {
    let tasks = tasks(ledger);
    let mut errors = duplicate_errors(&tasks);
    let numbers = own_numbers(&tasks, &ledger.prefix, &mut errors);
    let missing = missing_ranges(&numbers);
    if !missing.is_empty() {
        errors.push(format!(
            "missing task ids: {}",
            missing
                .into_iter()
                .map(|(start, end)| format_range(&ledger.prefix, start, end))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    errors
}

pub(super) fn highest_number(ledger: &Ledger) -> u32 {
    tasks(ledger)
        .into_iter()
        .filter_map(|task| number(task.id, &ledger.prefix))
        .max()
        .unwrap_or(0)
}

fn tasks(ledger: &Ledger) -> Vec<Task<'_>> {
    ledger
        .queue
        .iter()
        .map(|task| Task {
            id: &task.id,
            status: Status::Queue,
        })
        .chain(ledger.archive.iter().map(|task| Task {
            id: &task.id,
            status: Status::Archive,
        }))
        .chain(ledger.horizon.iter().map(|task| Task {
            id: &task.id,
            status: Status::Horizon,
        }))
        .collect()
}

fn duplicate_errors(tasks: &[Task<'_>]) -> Vec<String> {
    let mut claims: BTreeMap<&str, Vec<Status>> = BTreeMap::new();
    for task in tasks {
        claims.entry(task.id).or_default().push(task.status);
    }
    claims
        .into_iter()
        .filter_map(|(id, mut statuses)| {
            if statuses.len() == 1 {
                return None;
            }
            statuses.sort_unstable();
            let distinct: BTreeSet<Status> = statuses.iter().copied().collect();
            if distinct.len() == 1 {
                Some(format!(
                    "duplicate id {id} appears {} times in {}",
                    statuses.len(),
                    statuses[0]
                ))
            } else {
                Some(format!(
                    "id {id} has multiple statuses: {}",
                    distinct
                        .into_iter()
                        .map(|status| status.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            }
        })
        .collect()
}

fn own_numbers(tasks: &[Task<'_>], prefix: &str, errors: &mut Vec<String>) -> Vec<u32> {
    let mut numbers = BTreeSet::new();
    for task in tasks {
        let Some(number) = number(task.id, prefix) else {
            continue;
        };
        if number == 0 {
            errors.push(format!(
                "{} uses zero; the task sequence starts at {prefix}-001",
                task.id
            ));
            continue;
        }
        if number > MAX_TASK_NUMBER {
            errors.push(format!(
                "{} is outside the id space (max {prefix}-{MAX_TASK_NUMBER})",
                task.id
            ));
            continue;
        }
        let canonical = format!("{prefix}-{number:03}");
        if task.id != canonical {
            errors.push(format!("{} is not canonical; use {canonical}", task.id));
            continue;
        }
        numbers.insert(number);
    }
    numbers.into_iter().collect()
}

fn format_range(prefix: &str, start: u32, end: u32) -> String {
    if start == end {
        format!("{prefix}-{start:03}")
    } else {
        format!("{prefix}-{start:03}..{prefix}-{end:03}")
    }
}

fn number(id: &str, prefix: &str) -> Option<u32> {
    id.strip_prefix(prefix)?.strip_prefix('-')?.parse().ok()
}

fn missing_ranges(numbers: &[u32]) -> Vec<(u32, u32)> {
    let mut missing = Vec::new();
    let mut previous = 0;
    for number in numbers {
        if *number > previous + 1 {
            missing.push((previous + 1, number - 1));
        }
        previous = *number;
    }
    missing
}

#[cfg(test)]
mod tests {
    use super::{format_range, missing_ranges};

    #[test]
    fn missing_ids_are_complete_compact_ranges() {
        assert_eq!(missing_ranges(&[1, 3, 7, 8, 10]), [(2, 2), (4, 6), (9, 9)]);
        assert_eq!(format_range("QCTL", 4, 6), "QCTL-004..QCTL-006");
    }
}
