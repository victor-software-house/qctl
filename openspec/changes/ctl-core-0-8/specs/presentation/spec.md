## ADDED Requirements

### Requirement: Piped output defaults to tsv

When stdout is not a terminal and no flag or variable names a format, qctl SHALL write `tsv`.
A JSON failure SHALL go to stderr, and stdout SHALL stay empty after a failed run.

#### Scenario: Status piped

- **WHEN** `qctl status | cat` runs with no `--format` and no `QCTL_FORMAT`
- **THEN** each line holds tab-separated columns and no box-drawing characters

## MODIFIED Requirements

### Requirement: Piped output stays within the fallback width

qctl SHALL lay pretty output out to ctl-core's 80-column fallback when neither
stdout nor `COLUMNS` gives a width.

#### Scenario: Long title in a piped show

- **WHEN** `qctl show QCTL-001 --format pretty` runs with stdout piped and `COLUMNS` unset, and the row title is 104 characters
- **THEN** every output line is at most 80 columns wide
- **AND** the title wraps onto a second line
