# Name why a horizon row waits

## Why

A horizon row carries two fields that keep it off the queue.
`open` names what would end the wait, and `kind` says why the row waits: `research`, `evaluation`, or `deferred`.
`kind` names a category without saying what varies, which is the "kind as a noun" pattern the prose rules reject.
A reader has to learn from the docs that the field is the reason the row is not startable.

Measured at the head of `main` on 2026-11-09:

1. Three verbs take the flag: `add --horizon`, `park`, and `edit`, each as `-k/--kind`.
2. The field reaches four operator surfaces: the stored key, `status --format json`, the `status` horizon table column, and the error messages of `add` and `edit`.
3. This repository's ledger holds 2 horizon rows, each with a `kind:` key.

The operator chose `reason` on 2026-11-09, because the field says why a row waits on the horizon, beside `open`, which names what would end the wait.

Queue row: none yet. The open ledger pull requests hold the next ids, so the row that names this change is filed once they merge.

## What changes

1. **BREAKING**: a horizon row stores `reason:` instead of `kind:`, with the same values. `schema_version` moves from 5 to 6, because every row type refuses fields it does not know.
2. `qctl fmt` upgrades a schema 5 ledger to 6 by renaming each horizon row's `kind` key to `reason` where it is written, keeping its value, quotes, and comments. It upgrades schema 3 and 4 ledgers the same way, after the steps it already takes for them.
3. Every other verb refuses a schema 3, 4, or 5 ledger, writing nothing, and says to run `qctl fmt`. No reader accepts `kind` outside that upgrade, and there is no alias.
4. **BREAKING**: `-r/--reason` replaces `-k/--kind` on `add`, `park`, and `edit`. The variable follows the prefix rule, so `QCTL_KIND` becomes `QCTL_REASON`. `-r` is free on all three verbs; `close-from-git` keeps its own `-r/--range`.
5. JSON output, the `status` horizon table, help text, and error messages say `reason`.
6. Other operator text that used "kind" as a noun names what varies instead.

## Capabilities

### New Capabilities

1. `horizon-rows`: what a horizon row stores about why it waits, the flags that write it, and how `fmt` upgrades a ledger that still says `kind`.

### Modified Capabilities

1. `openspec-agreement`: its schema version 5 requirement is removed, because `horizon-rows` now states the current version and every upgrade to it.

## Impact

1. Code: `src/schema.rs` (`HorizonTask::reason`, the `Reason` enum, `VERSION`), `src/cli.rs`, `src/mutate.rs`, `src/edit/`, `src/presentation.rs`, `src/document.rs` (the in-place key rename), `src/document/scalars.rs`, `src/format.rs`, and `src/ledger.rs` (the refusal messages).
2. Schema: `schema/tasks.schema.json` names `reason` and the `Reason` definition, with `schema_version` 6.
3. Contracts: `src/instructions.md`, `skills/qctl/SKILL.md`, the templates under `.ctl/operator/`, `AGENTS.md`, and `docs/adoption-report.md`.
4. Tests: every test ledger moves to version 6 and `reason:`, the mutation snapshots that show those lines, the usage snapshot, and new upgrade tests for schema 5.
5. Consumers: every ledger runs `qctl fmt` once in the pull request that takes the release and moves its `$schema` pin. A caller that set `QCTL_KIND` or passed `-k` changes to `QCTL_REASON` or `-r`.
