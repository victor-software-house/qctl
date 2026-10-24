# Check that queue rows and OpenSpec changes agree

## Why

A repository that plans with OpenSpec keeps two records of the same work: the change under `openspec/changes/<name>/`, and the qctl row that tracks it.
A row names its change through its `plan:` path, and people archive the two by hand.
Nothing checks that the link is true, so the records drift.

Measured on 2026-10-23 at the head of the seven ctl ledgers that sit beside an `openspec/` tree, six public and one private:

1. Finished rows whose change was never archived: 7 when a row is matched by the change name anywhere in the ledger, 4 when only `plan:` counts.
   In the public ledgers, qctl's QCTL-045, ctl-core's CTC-022, and verctl's VER-067 finished on 2026-10-13 to 2026-10-17, and their changes are still open.
2. Open changes that no row names: 8 by name anywhere, 18 by `plan:` only.
   Most rows name their change in a note, not in `plan:`, so the 18 includes changes that are tracked, just not linked.
3. A queued row whose change was already archived: 1 (ctl-core CTC-008).
4. One change named by several rows: verctl's `release-state-machine` is named by five rows, two queued and three finished.
   This repository's `openspec/config.yaml` says one change maps to one row, and that does not hold.

qctl already has the model for closing rows from another record: `close-from-git` archives the rows a commit trailer closed, and `check` fails while such a row is still queued.
OpenSpec 1.15.0 still has no extension point for outside checks ([#829][openspec-829] is open), so the agreement has to be checked beside it.

Queue row: QCTL-051.

## What changes

1. A row names an OpenSpec change when its `plan:` is a file inside `openspec/changes/<name>/`, or inside `openspec/changes/archive/<date>-<name>/`.
   There is no new link field.
   One change may be named by several rows; a row names at most one change.
2. `qctl check` reads the `openspec/` tree next to the ledger and reports, as problems:
   1. a finished row whose change is still open, when no queued or horizon row names the same change and the row states no waiver;
   2. a row whose `plan:` points into an open change that was since archived, naming the archive folder and the verb that fixes it;
   3. an open change that no row names.
3. `qctl archive` refuses a row whose change is still open when no other queued or horizon row names it, unless `--waiver` gives the reason.
   The reason is stored on the archived row as `waiver`.
4. **BREAKING**: `schema_version` moves from 4 to 5, because archived rows may carry `waiver` and every row type refuses fields it does not know.
   Every verb but `fmt` refuses a schema 4 ledger and says to run `qctl fmt`, which sets `schema_version: 5` and changes nothing else.
5. A new verb, `qctl close-from-openspec`, archives each queued row whose change was archived, with the archive folder as evidence, and points every row that named the change at the archived file.
6. The agreement rules run only when an `openspec/` directory sits next to the ledger. Where it does, a mismatch is an error, and a row's waiver is the only way past it; there is no flag to turn the rules off.
7. qctl reads the folder layout itself. It never runs `openspec` or Node.
8. This repository's `openspec/config.yaml` and `AGENTS.md` say that one change may serve several rows.

## Capabilities

### New Capabilities

1. `openspec-agreement`: how qctl links a row to an OpenSpec change, which disagreements `check` reports, how `archive` gates on the change, and what `close-from-openspec` writes.

### Modified Capabilities

None.

## Impact

1. Code: a new `src/openspec.rs` layout reader, `src/check.rs`, `src/ledger.rs` (the missing-plan message), `src/mutate.rs` (`archive` and the new verb), `src/cli.rs`, `src/report.rs`, `src/presentation.rs`, `src/schema.rs` (the optional `waiver` field).
2. Schema: archived rows gain an optional `waiver` string, and `schema_version` becomes 5. The published `$schema` URL moves with the release, and `qctl fmt` upgrades a schema 4 ledger.
3. Contracts: `src/instructions.md`, `skills/qctl/SKILL.md`, the operator templates under `.ctl/operator/`, `AGENTS.md`, `openspec/config.yaml`.
4. Tests: check, archive, and `close-from-openspec` scenarios over a scratch ledger with an `openspec/` tree.
5. Consumers: every ledger runs `qctl fmt` once in the pull request that takes the release, and moves its `$schema` pin. A ledger beside an `openspec/` tree also fails `check` until its rows and changes agree; each ctl repository fixes that drift in the same pull request.

[openspec-829]: https://github.com/Fission-AI/OpenSpec/issues/829
