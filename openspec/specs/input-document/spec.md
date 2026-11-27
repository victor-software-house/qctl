# input-document Specification

## Purpose

How a qctl command reads its arguments from a YAML or JSON document through `--input` and `--input-format`, and which source wins when the command line, the environment and the document all name the same argument.

## Requirements

### Requirement: A command reads its arguments from a document

Every qctl command SHALL take `--input <FILE>` and `--input-format <NAME>`, also read from `QCTL_INPUT` and `QCTL_INPUT_FORMAT`.
A document's keys SHALL be long option names and positional ids, and each value SHALL pass the argument's own parser.
The command line SHALL win over the environment, the environment over the document, and the document over the default.

#### Scenario: Add a row from a JSON file

- **WHEN** `qctl add --input row.json` runs where `row.json` holds a title, a scope, an outcome, two acceptance items and a note
- **THEN** `tasks.yaml` gains the same row as the matching `-t`, `-s`, `-o`, `-a` and `-n` flags write

#### Scenario: Edit a row from a YAML file

- **WHEN** `qctl edit QCTL-001 --input edit.yaml` runs where `edit.yaml` holds `title: New title`
- **THEN** `tasks.yaml` changes as `qctl edit QCTL-001 -t 'New title'` changes it
