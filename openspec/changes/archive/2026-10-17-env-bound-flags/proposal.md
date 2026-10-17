# Read every flag from the environment

## Why

ctl shared rule 7 says every CLI binds each long option to `<CLI>_<LONG>`, so a caller that holds settings in the environment sets `env:` and runs the bare verb.
verctl and forkctl do this through ctl-core's `EnvPrefix`; qctl does not.
qctl reads one variable of its own, `TASKS_LEDGER`, for `--file`, and nothing else, so `QCTL_FORMAT=json` has no effect.

Queue row: QCTL-049.

## What changes

1. The root parser carries `#[command(env_prefix = "QCTL")]`, so every long option reads `QCTL_<LONG>`: `QCTL_FILE`, `QCTL_FORMAT`, `QCTL_COLOR`, and each verb's own options.
2. `TASKS_LEDGER` is removed with no alias. `--file` reads `QCTL_FILE`, then defaults to `tasks.yaml`.
3. Help prints `[env: NAME]` beside each bound option, through ctl-core.
4. The operator docs name the rule and `QCTL_FILE`.

## Impact

1. `src/cli.rs`, `src/ledger.rs`, `tests/common/mod.rs`, `tests/cli.rs`, the Usage snapshot, operator docs, README, AGENTS.md.
2. A consumer that sets `TASKS_LEDGER` renames it to `QCTL_FILE` when it takes this release.
