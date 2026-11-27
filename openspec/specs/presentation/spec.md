# presentation Specification

## Purpose

How qctl presents ledger rows on top of the ctl-core presentation kernel: which values take the identifier role, and how piped output is laid out.

## Requirements

### Requirement: Piped output stays within the fallback width

qctl SHALL lay pretty output out to ctl-core's 80-column fallback when neither
stdout nor `COLUMNS` gives a width.

#### Scenario: Long title in a piped show

- **WHEN** `qctl show QCTL-001 --format pretty` runs with stdout piped and `COLUMNS` unset, and the row title is 104 characters
- **THEN** every output line is at most 80 columns wide
- **AND** the title wraps onto a second line

### Requirement: Row ids use the identifier role

qctl SHALL render row ids through ctl-core's identifier role and keep paths as
tokens.

#### Scenario: Status queue

- **WHEN** `qctl status --color always` lists the queue
- **THEN** each row id is bold with no colour escape
- **AND** the ledger path keeps the token colour

### Requirement: Piped output defaults to tsv

When stdout is not a terminal and no flag or variable names a format, qctl SHALL write `tsv`.
A JSON failure SHALL go to stderr, and stdout SHALL stay empty after a failed run.

#### Scenario: Status piped

- **WHEN** `qctl status | cat` runs with no `--format` and no `QCTL_FORMAT`
- **THEN** each line holds tab-separated columns and no box-drawing characters
