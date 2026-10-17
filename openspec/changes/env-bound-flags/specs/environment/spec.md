## ADDED Requirements

### Requirement: Every long option reads a QCTL variable

Each long option of every verb SHALL read `QCTL_<LONG>`, with the long name upper-cased and `-` written as `_`.
A flag on the command line SHALL win over its variable, and an empty variable SHALL count as unset.
Help SHALL print `[env: NAME]` beside each bound option.

#### Scenario: The format comes from the environment

- **WHEN** `qctl status --file tasks.yaml` runs with `QCTL_FORMAT=json`
- **THEN** stdout is JSON

#### Scenario: The command line wins

- **WHEN** `qctl status --file tasks.yaml --format pretty` runs with `QCTL_FORMAT=json`
- **THEN** stdout is not JSON

#### Scenario: The ledger comes from the environment

- **WHEN** `qctl status --format json` runs in a directory with no `tasks.yaml`, with `QCTL_FILE=other/tasks.yaml`
- **THEN** it reads `other/tasks.yaml`

### Requirement: TASKS_LEDGER is not read

qctl SHALL NOT read `TASKS_LEDGER`.

#### Scenario: The old variable is ignored

- **WHEN** `qctl status` runs with `TASKS_LEDGER=other/tasks.yaml` and no `tasks.yaml` in the directory
- **THEN** it fails naming `tasks.yaml`
