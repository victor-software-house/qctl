# horizon-rows Specification

## Purpose
What a horizon row stores about why it waits, the flags that write it, and how `qctl fmt` upgrades a ledger that still says `kind`.

## Requirements

### Requirement: A horizon row says why it waits in reason

A horizon row SHALL carry `reason`, with the values `research`, `evaluation`, and `deferred`, and `open`, which names what would end the wait.
qctl SHALL refuse a horizon row without `reason`, with another value, or with a `kind` key.
`status --format json` and `show --format json` SHALL name the field `reason`, and the `status` horizon table SHALL head its column `reason`.

#### Scenario: A horizon row without a reason is refused

- **WHEN** a horizon row in `tasks.yaml` has `open` and no `reason`
- **THEN** `qctl check --no-git` reports a problem that names `reason`

#### Scenario: An unknown reason is refused

- **WHEN** a horizon row in `tasks.yaml` has `reason: someday`
- **THEN** `qctl check --no-git` reports a problem that names `reason`

#### Scenario: JSON names the field

- **WHEN** horizon row `QCTL-002` has `reason: research`
- **AND** the operator runs `qctl status --format json`
- **THEN** `ledger.horizon[0].reason` is `"research"`

### Requirement: The verbs write reason through -r/--reason

`qctl add --horizon`, `qctl park`, and `qctl edit` SHALL take the reason as `-r/--reason`, read `QCTL_REASON` when the flag is absent, and SHALL NOT accept `-k` or `--kind`.
`add --horizon` and `park` SHALL require it.
`promote` SHALL drop `reason` and `open` from the row it moves onto the queue.

#### Scenario: Parking a row with a reason

- **WHEN** the operator runs `qctl park QCTL-001 -r research -O 'The benchmark result is missing.'`
- **THEN** the row moves to the horizon with `reason: research` and `open: The benchmark result is missing.`

#### Scenario: The variable sets the reason

- **WHEN** `QCTL_REASON=deferred` is set
- **AND** the operator runs `qctl park QCTL-001 -O 'The budget is not approved.'`
- **THEN** the parked row has `reason: deferred`

#### Scenario: The old flag is refused

- **WHEN** the operator runs `qctl park QCTL-001 --kind research -O wait`
- **THEN** the command fails as an unknown argument and the ledger is unchanged

### Requirement: A ledger declares schema version 6

qctl SHALL write and accept `schema_version: 6`.
`qctl fmt` SHALL upgrade a schema 3, 4, or 5 ledger to 6.
It SHALL rename each horizon row's `kind` key to `reason` where the key is written, keeping the value, its quotes, and every comment, and SHALL set `schema_version: 6`.
No other path SHALL read `kind`.

#### Scenario: Upgrading a schema 5 ledger in place

- **WHEN** `tasks.yaml` declares `schema_version: 5`, is in its declared style, and has a horizon row written `kind: research # until the benchmark runs`
- **AND** the operator runs `qctl fmt`
- **THEN** that line reads `reason: research # until the benchmark runs`
- **AND** the only other changed lines are the other horizon rows' `kind:` keys, now `reason:`, and `schema_version: 6`

#### Scenario: Upgrading a schema 4 ledger with no horizon rows

- **WHEN** `tasks.yaml` declares `schema_version: 4` and `horizon: []`
- **AND** the operator runs `qctl fmt`
- **THEN** the only changed line is `schema_version: 6`

### Requirement: Other verbs refuse a ledger older than schema 6

Every verb but `fmt` SHALL refuse a schema 4 or 5 ledger, writing nothing, with `tasks.yaml: schema_version N; run qctl fmt to rename the horizon key kind to reason and set schema_version 6`.
`qctl check` SHALL report that line as its only problem.

#### Scenario: A verb on a schema 5 ledger

- **WHEN** `tasks.yaml` declares `schema_version: 5`
- **AND** the operator runs `qctl status`
- **THEN** the command fails with `tasks.yaml: schema_version 5; run qctl fmt to rename the horizon key kind to reason and set schema_version 6`
- **AND** `qctl check --no-git` reports that line as its only problem
