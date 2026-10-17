# Design

## Decisions

### 1. Bind through ctl-core

`#[command(env_prefix = "QCTL")]` on `Cli`, with `ctl_core::EnvPrefix` in scope.
ctl-core's `App` already unbinds a variable the command line overrules before the parse, so the precedence is flag, then variable, then default.

Alternative: an `env = "..."` attribute on each flag.
It lost because every new flag would need its own line, and rule 7 says one attribute on the root.

### 2. Remove `TASKS_LEDGER` without an alias

`resolve_path` keeps `--file`, which now carries `QCTL_FILE`, and the `tasks.yaml` default.

Alternative: read `TASKS_LEDGER` when `QCTL_FILE` is unset.
It lost because two names for one setting is the drift rule 7 removes, and the known consumers rename in their own pull requests when they take the release.

### 3. Override and disable path

A flag on the command line wins over its variable, and an empty variable is unset.
Unsetting the variable disables it; no option turns the binding off.

## Risks

1. [A consumer still sets `TASKS_LEDGER`] → qctl then reads `tasks.yaml` in the working directory. The changeset names the rename, and each known consumer gets a rename pull request from its owner.
