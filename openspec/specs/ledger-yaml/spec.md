# ledger-yaml Specification

## Purpose

How qctl reads a ledger and where it reports a problem, and how an edit
keeps every byte it does not name.

## Requirements

### Requirement: Ledgers are read through the shared declared input

qctl SHALL parse every ledger through ctl-core's `input` feature, and SHALL
report each parse or value problem with the ledger's file name and line.

#### Scenario: A value error names its line

- **WHEN** `tasks.yaml` has `prefix: q` on line 2 and the operator runs
  `qctl status`
- **THEN** the command fails and the error names `tasks.yaml:2:9: prefix:`

#### Scenario: A duplicate key is refused

- **WHEN** a queue row in `tasks.yaml` writes `title:` twice
- **THEN** `qctl status` fails and names the line of the second `title:`

### Requirement: Schema errors from check are placed on their lines

`qctl check` SHALL prefix each JSON Schema error with the file, line, and
column of the node its instance path names, or of that node's nearest written
ancestor.

#### Scenario: A wrong id is placed

- **WHEN** `queue[0].id` in `tasks.yaml` is `Q-1`, written on line 5, column 9
- **THEN** `qctl check --no-git` reports a problem starting with
  `tasks.yaml:5:9: /queue/0/id:`

### Requirement: Edits keep every byte they do not name

qctl SHALL write each ledger edit through yamled, so bytes outside the edited
nodes, comments included, stay as they were.

#### Scenario: Starting a row moves only that row

- **WHEN** the operator runs `qctl start QCTL-005` on a ledger whose rows carry
  comments
- **THEN** the written file differs from the original only in the moved row's
  position and the `active` value, and each comment stays with the row it was
  written above

### Requirement: A comment that belongs to no list stays in its slot

`qctl fmt` SHALL reorder the lists around a comment that has a blank line under
it, leaving that comment where it was written, and SHALL move a comment written
directly above a list key with that list.

#### Scenario: Lists trade places around a loose comment

- **WHEN** `tasks.yaml` declares `section_order: [queue, horizon, archive]`,
  writes `queue`, a comment with a blank line under it, `archive`, then
  `horizon`, and the operator runs `qctl fmt`
- **THEN** the file lists `queue`, the comment, `horizon`, then `archive`

### Requirement: qctl carries no YAML crate of its own

qctl SHALL depend on no YAML parser or editor other than ctl-core's `input`
feature and yamled.

#### Scenario: The dependency tree is checked

- **WHEN** the build host runs `cargo tree -e normal` for qctl
- **THEN** the output names none of `serde_yml`, `yaml_serde`, `yamlpath`,
  `yamlpatch`, or a `tree-sitter` crate
