# Spec Delta

## Purpose

How qctl links a queue row to an OpenSpec change through the row's `plan:` path, which disagreements between rows and changes `qctl check` reports, how `qctl archive` gates on the change, and what `qctl close-from-openspec` writes.

## ADDED Requirements

### Requirement: A row names a change through its plan path

qctl SHALL treat a row as naming change `<name>` when the row's `plan:` is a file under `openspec/changes/<name>/` or under `openspec/changes/archive/<YYYY-MM-DD>-<name>/`, both relative to the ledger's directory.
qctl SHALL treat a row with any other `plan:`, or none, as naming no change.
Several rows MAY name the same change.

#### Scenario: An open change named by plan

- **WHEN** `DEMO-001` has `plan: openspec/changes/add-thing/proposal.md`
- **THEN** `DEMO-001` names the open change `add-thing`

#### Scenario: An archived change named by plan

- **WHEN** `DEMO-002` has `plan: openspec/changes/archive/2026-10-20-add-thing/proposal.md`
- **THEN** `DEMO-002` names the archived change `add-thing`

#### Scenario: Other plans name no change

- **WHEN** `DEMO-003` has `plan: goals/row-edit/plan.md`
- **THEN** `DEMO-003` names no change, and no agreement rule reads it

### Requirement: check reports a finished row whose change is still open

When an `openspec/` directory sits next to the ledger, `qctl check` SHALL report a problem for each archived row that names an open change, unless a queued or horizon row names the same change or the archived row has a non-empty `waiver`.

#### Scenario: The last finished row leaves its change open

- **WHEN** `DEMO-004` is archived with `plan: openspec/changes/add-thing/proposal.md`
- **AND** `openspec/changes/add-thing/` exists and no queued or horizon row names `add-thing`
- **THEN** `qctl check` exits non-zero
- **AND** its problems include `DEMO-004: change add-thing is still open; archive it with openspec archive, then run qctl close-from-openspec`

#### Scenario: A shared change stays open while a row still needs it

- **WHEN** `DEMO-004` is archived and `DEMO-005` is queued, both with `plan: openspec/changes/add-thing/proposal.md`
- **THEN** `qctl check` reports no problem about `add-thing`

#### Scenario: A waiver accepts the open change

- **WHEN** `DEMO-004` is archived with `waiver: The change continues under another ledger.` and names the open change `add-thing`
- **THEN** `qctl check` reports no problem about `add-thing`

### Requirement: check names the archive of a change a plan still points into

When a row's `plan:` points into `openspec/changes/<name>/`, that file is missing, and exactly one `openspec/changes/archive/<date>-<name>/` holds a file of the same name, the `missing plan` problem SHALL name that archive folder and `qctl close-from-openspec`.
When more than one archive folder ends in `-<name>`, the problem SHALL say the archive is ambiguous and name every folder.

#### Scenario: A queued row after openspec archive

- **WHEN** `DEMO-005` is queued with `plan: openspec/changes/add-thing/proposal.md`
- **AND** `openspec archive add-thing` moved the change to `openspec/changes/archive/2026-10-23-add-thing/`
- **THEN** `qctl check` reports `missing plan openspec/changes/add-thing/proposal.md; change add-thing was archived to openspec/changes/archive/2026-10-23-add-thing/; run qctl close-from-openspec`

### Requirement: check reports an open change that no row names

When an `openspec/` directory sits next to the ledger, `qctl check` SHALL report a problem for each directory under `openspec/changes/` other than `archive` that no queued, archived, or horizon row names.

#### Scenario: A change tracked only in a note

- **WHEN** `openspec/changes/toml-input/` exists and a row mentions `toml-input` only in its notes
- **THEN** `qctl check` reports `change toml-input is named by no row; point a row's plan at openspec/changes/toml-input/proposal.md`

### Requirement: The agreement rules can be turned off for one run

`qctl check --no-openspec`, and `QCTL_NO_OPENSPEC=true`, SHALL skip every rule of this capability for that run.
A ledger with no `openspec/` directory next to it SHALL get none of these rules.

#### Scenario: A repository that cannot fix its drift yet

- **WHEN** a ledger has a finished row whose change is open
- **AND** the operator runs `qctl check --no-openspec`
- **THEN** no agreement problem is reported, and every other rule still runs

#### Scenario: A ledger without OpenSpec

- **WHEN** no `openspec/` directory sits next to `tasks.yaml`
- **THEN** `qctl check` reports no agreement problem

### Requirement: archive refuses the last row of an open change without a waiver

`qctl archive ID` SHALL refuse, writing nothing, when the row names an open change, no other queued or horizon row names it, and no `--waiver` is given.
With `--waiver REASON`, or `QCTL_WAIVER`, it SHALL archive the row and write `waiver: REASON` on it.
`qctl close-from-git` SHALL apply the same gate to each row a trailer closes, archive the rows that pass, and report the refused ids.

#### Scenario: Archiving before the change

- **WHEN** `DEMO-005` is the only row naming the open change `add-thing`
- **AND** the operator runs `qctl archive DEMO-005 -e https://example.com/pull/1`
- **THEN** the command fails with `DEMO-005 names open change add-thing; archive the change first (openspec archive add-thing, then qctl close-from-openspec), or pass --waiver REASON`
- **AND** `tasks.yaml` is unchanged

#### Scenario: Archiving with a waiver

- **WHEN** the operator runs `qctl archive DEMO-005 -e https://example.com/pull/1 --waiver "The change continues under another ledger."`
- **THEN** `DEMO-005` is archived with `waiver: The change continues under another ledger.`

#### Scenario: Archiving one row of a shared change

- **WHEN** `DEMO-004` and `DEMO-005` are queued and both name the open change `add-thing`
- **AND** the operator runs `qctl archive DEMO-004 -e https://example.com/pull/1`
- **THEN** `DEMO-004` is archived and keeps `plan: openspec/changes/add-thing/proposal.md`

### Requirement: close-from-openspec closes and repoints rows of archived changes

`qctl close-from-openspec` SHALL archive each queued row whose open-change `plan:` is missing and whose change has exactly one archive folder, with that folder as its only evidence and disposition `completed`.
It SHALL point the `plan:` of every row it archived, and of every archived row in the same state, at the same file name inside the archive folder.

#### Scenario: Closing after openspec archive

- **WHEN** `DEMO-005` is queued with `plan: openspec/changes/add-thing/proposal.md`
- **AND** the change was moved to `openspec/changes/archive/2026-10-23-add-thing/`
- **AND** the operator runs `qctl close-from-openspec`
- **THEN** `DEMO-005` is archived with evidence `openspec/changes/archive/2026-10-23-add-thing/` and `plan: openspec/changes/archive/2026-10-23-add-thing/proposal.md`
- **AND** `qctl check` reports no agreement problem

#### Scenario: Repointing a row that finished earlier

- **WHEN** `DEMO-004` was archived with `plan: openspec/changes/add-thing/proposal.md` while `DEMO-005` still named the change
- **AND** `qctl close-from-openspec` runs after the change was archived
- **THEN** `DEMO-004` gets `plan: openspec/changes/archive/2026-10-23-add-thing/proposal.md` and keeps its evidence and `completed`

#### Scenario: Nothing to close

- **WHEN** every row's plan resolves
- **THEN** `qctl close-from-openspec` writes nothing and reports no archived or repointed ids

### Requirement: close-from-openspec leaves what it cannot decide

`qctl close-from-openspec` SHALL leave horizon rows and rows with an ambiguous archive unchanged and list them in its report.
It SHALL refuse, writing nothing, when `active` names a row it would archive.

#### Scenario: A parked row whose change was archived

- **WHEN** horizon row `DEMO-006` has `plan: openspec/changes/add-thing/proposal.md` and the change was archived
- **THEN** `qctl close-from-openspec` leaves `DEMO-006` unchanged and lists it as left for the operator

#### Scenario: The active row would close

- **WHEN** `active: DEMO-005` and `DEMO-005` names a change that was archived
- **THEN** `qctl close-from-openspec` fails with `active DEMO-005 names archived change add-thing; set active to another row first`
- **AND** `tasks.yaml` is unchanged
