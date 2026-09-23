# Presentation

## ADDED Requirements

### Requirement: Piped output stays within the fallback width

qctl SHALL lay pretty output out to ctl-core's 80-column fallback when neither
stdout nor `COLUMNS` gives a width.

#### Scenario: Long title in a piped show

- **WHEN** `qctl show QCTL-001` runs with stdout piped and `COLUMNS` unset, and the row title is 104 characters
- **THEN** every output line is at most 80 columns wide
- **AND** the title wraps onto a second line

### Requirement: Row ids use the identifier role

qctl SHALL render row ids through ctl-core's identifier role and keep paths as
tokens.

#### Scenario: Status queue

- **WHEN** `qctl status --color always` lists the queue
- **THEN** each row id is bold with no colour escape
- **AND** the ledger path keeps the token colour
