# Design

## Context

See `proposal.md` for the measured drift.
Today a row's `plan:` is a path next to the ledger, and `check` only confirms the file exists (`graph_errors` in `src/ledger.rs`).
Rows that follow the OpenSpec convention point `plan:` at `openspec/changes/<name>/proposal.md`.
After `openspec archive` moves the folder to `openspec/changes/archive/<YYYY-MM-DD>-<name>/`, `edit ID -l PATH` points an archived row at the moved file (QCTL-042).
`close-from-git` archives queued rows named by commit trailers, and `check` reports the same rows while they stay queued.

## Goals / Non-Goals

**Goals:**

1. `check` fails when a row and its change disagree, in either direction.
2. A finished row cannot leave its change open by accident.
3. One verb brings rows in line after `openspec archive`, as `close-from-git` does after a commit.

**Non-Goals:**

1. Checking what a change says: proposals, specs, and tasks stay `openspec validate`'s business.
2. Running `openspec`. qctl gains no runtime dependency.
3. Decision records and research notes. A separate document checker will cover them.
4. Writing anything into OpenSpec's files, such as a back-link in `.openspec.yaml`. OpenSpec 1.14.2 ignores `x-` keys there but calls that "not an officially supported extension mechanism".
5. Hook wiring for `close-from-openspec`. The verb runs in the same pull request as `openspec archive`; `check` names it when it is due.

## Decisions

### 1. The link is the `plan:` path

A row names change `<name>` when its `plan:` is a file under `openspec/changes/<name>/` (an open change) or under `openspec/changes/archive/<date>-<name>/` (an archived one), where `<date>` is `YYYY-MM-DD` and `<name>` is not `archive`.
Any other `plan:`, or none, names no change.
The `openspec/` root is the directory next to the ledger, the same base `plan:` already resolves against.
A row that mentions a change only in a note does not name it.

Alternative: a new `change:` field.
It lost because `plan:` already carries the name, QCTL-042 keeps it true after archive, and a field would need a schema migration and a second fact to keep in step.

### 2. One change may serve several rows

Several rows may name one change; a row names at most one, because it has one `plan:`.
A change stays legitimately open while any queued or horizon row names it.
A finished row is in agreement when its change is archived, when another queued or horizon row still names the open change, or when the row gives a `keep_change_open` reason.

Alternative: one row per change, as `openspec/config.yaml` said.
It lost on evidence: verctl's `release-state-machine` serves five rows, and splitting a design across five changes would only move the link problem into OpenSpec.

### 3. The rules `check` adds

When the `openspec/` directory exists, `check` reports:

1. **Finished row, open change.** An archived row names an open change, no queued or horizon row names it, and the row has no `keep_change_open`: `QCTL-045 is done but its change plain-json-forced-colour is still open; archive the change with openspec archive and run qctl close-from-openspec, or set keep_change_open with a reason`.
2. **Plan points into an archived change.** A row's `plan:` points into `openspec/changes/<name>/`, that file is missing, and exactly one `openspec/changes/archive/<date>-<name>/` holds the same file name: the existing `missing plan` problem gains `; change <name> was archived to <folder>; run qctl close-from-openspec`.
3. **Change no row names.** An open change that no row names: `change toml-input is named by no row; point a row's plan at openspec/changes/toml-input/proposal.md`.

When two archive folders end in the same `-<name>`, rule 2 says the archive is ambiguous and names both; `close-from-openspec` leaves those rows alone.
A horizon row may name an open change: a parked row keeps its plan, and its change waits with it.
Every rule reports through the existing `problems` list, so `--format json` and the exit code carry it unchanged.

Alternative: report these as warnings.
It lost because `check` has one severity and is what fails hooks and CI; a warning nobody fails on is how the drift above built up.
A row's `keep_change_open` (decision 5) is the only override (decision 7).

### 4. `archive` refuses the last row of an open change

`qctl archive ID` refuses when the row names an open change, no other queued or horizon row names it, and `--keep-change-open` is absent:
`QCTL-051 is the last row naming open change openspec-agreement; archive the change first (openspec archive openspec-agreement, then qctl close-from-openspec), or pass --keep-change-open REASON`.
When another row still names the change, `archive` proceeds and the row keeps its `plan:`, which still resolves.
`close-from-git` goes through the same refusal, so a trailer cannot close a row whose change is open; it reports the refused id and archives the rest.

### 5. `keep_change_open` holds the reason on the archived row

`qctl archive --keep-change-open REASON` (`QCTL_KEEP_CHANGE_OPEN`) writes `keep_change_open: REASON` on the archived row.
`ArchivedTask` gains an optional `keep_change_open` string, documented as the reason the row's change stays open after the row finished.
`check` accepts a finished row with an open change when its `keep_change_open` is present and not empty.

The field moves `schema_version` to 5 (decision 9).

Alternative: treat `disposition: dropped` as a reason to keep the change open.
It lost because dropping a row says nothing about its change, which may still be wanted by another ledger or need deleting.

Alternative: a note with a fixed prefix.
It lost because a note is free text, and `check` would have to parse prose to find a rule.

### 6. `close-from-openspec` archives and repoints

`qctl close-from-openspec` reads the archive folders once, then:

1. archives each queued row whose open-change `plan:` is missing and whose change has exactly one archive folder, with evidence `openspec/changes/archive/<date>-<name>/` and disposition `completed`, through the same path as `archive`;
2. points `plan:` at the archived file for every row it archived and every archived row in the same state, as `edit ID -l` would;
3. refuses to run when `active` names a row it would archive, as `close-from-git` does.

A horizon row whose change was archived is reported and left alone; the operator decides whether to promote and close it or drop it.
The report lists the archived ids and the repointed ids; `--format json` carries both.
`--range` and `--pre-push` have no meaning here, because the archive folder is the evidence, not a commit.

Alternative: fold this into `close-from-git`.
It lost because the two read different records, and `close-from-git` runs in pre-push, where an archive folder may not be committed yet.

### 7. `keep_change_open` is the only override

`archive --keep-change-open` overrides the refusal for one row and records why, and the same `keep_change_open` satisfies `check` rule 1.
A ledger with no `openspec/` directory next to it gets none of these rules, because it has no changes to agree with.
Where the directory exists, nothing turns the rules off.

Alternative: `check --no-openspec`, like `--no-git`.
It lost because a flag that turns the rules off for a whole run becomes the habitual way around them, and leaves no record of why. `--no-git` exists for a different reason: a scratch ledger outside the repository has no history to scan, while a ledger beside `openspec/` always has changes to compare.

### 8. The layout reader lives in qctl for now

`src/openspec.rs` reads the layout: open change names, archive folders by name, and the change a `plan:` names.
It reads directory names only, never file contents, so OpenSpec releases that change spec syntax do not touch it.
It returns typed values to `check` and the verbs and prints nothing.

Alternative: a ctl-core feature shared with the future document checker.
It lost for now because qctl is its only reader, and an abstraction for one use is drift waiting for its second user. It moves to ctl-core in the change that adds that second reader.

### 9. `keep_change_open` moves the schema to version 5

Every row type is `#[serde(deny_unknown_fields)]`, and `Ledger::schema_version` is the one version a qctl writes and accepts.
A schema 4 qctl reading a row with `keep_change_open` fails on the field, so a ledger carrying it would break every older qctl with an error that does not name the cause.
Moving to version 5 makes the older qctl say `schema_version 5 must be 4` instead, which tells the reader to upgrade qctl.

What each verb does, following the version 3 to 4 precedent:

1. `qctl fmt` upgrades a schema 4 ledger by setting `schema_version: 5` and changes nothing else, because version 5 only adds an optional field. A schema 3 ledger is rewritten to 4 as today, then set to 5.
2. Every other verb refuses a schema 4 ledger: `tasks.yaml: schema_version 4; run qctl fmt to set schema_version 5`.
3. `qctl check` reports that line as its only problem, as it does for schema 3.
4. `qctl init` writes `schema_version: 5`, and `qctl schema` publishes the version 5 schema at the release's pinned `$schema` URL.

Alternative: keep version 4, as the `style` options `list_items` and `quotes` did.
It lost because those options already break an older qctl with a field error on any ledger that declares them; this change does not repeat that. It does not revisit those options either.

Alternative: accept both 4 and 5, and allow `keep_change_open` only under 5.
It lost because `Ledger` states one accepted version, and two would mean a version check per field.

## Risks / Trade-offs

1. [Ledgers fail `check` on the first run after the release] → Each ctl repository fixes its drift in the pull request that takes the release, which is how the rules prove themselves on real data. A repository that cannot fix it yet stays on the previous qctl.
2. [Every consumer must run `qctl fmt` and move its `$schema` pin] → The upgrade is one line per ledger, made in the same pull request that bumps qctl.
3. [Most rows name their change in a note today] → Rule 3 reports those changes until a row's `plan:` names them. The fix is one `qctl edit ID -l PATH` per change, listed in the rollout.
4. [OpenSpec changes its archive naming] → The reader is one module, and the layout has not changed from 1.13.2 to 1.15.0.
5. [A `keep_change_open` hides drift] → The reason is in the file, in review, and in `show`.

## Migration Plan

1. qctl ships this change as a patch, per the house rule.
2. Each consumer bumps qctl, runs `qctl fmt` to reach schema 5, and moves its `$schema` pin.
3. Each ctl repository beside an `openspec/` tree then runs `qctl check` and fixes what it reports: `openspec archive` for finished work, then `qctl close-from-openspec`; `qctl edit ID -l PATH` for changes tracked only in notes.
4. Rollback: revert the qctl pin, remove any `keep_change_open` field, and set `schema_version: 4` by hand. Everything else the new verbs write is valid for the older release.
