# QCTL-030 — update a row through existing verbs, ctl-core first

One qctl patch release, gated behind one ctl-core patch release. Recorded
2026-08-26 after implementation was stopped for violating that order.

## Operator intent

- Metadata on an existing task must be updatable by a real command. `add --notes`
  is create-time only, and a blessed YAML splice is not an answer.
- The API must be intuitive and complete, with few stems. Prefer flags and
  parameters over new top-level verbs.
- `notes` behaves like `acceptance`: a list, repeatable, removable, reorderable —
  on queued, horizon, and archived rows.
- Every operator parameter has a collision-free short option. `-f` stays qctl's
  ledger-file shorthand; ctl-core keeps its output/color shorts.
- ctl-core supplies the smallest reusable assertion so a long option cannot ship
  without a short. No framework.
- Patch bump for both packages. Never resurrect `task-ledger`. Never
  `--no-verify`. Never merge either PR without the operator saying so.

## Contract this slice ships

- `schema_version: 4`; `notes: Vec<String>` on all three row types.
- `fmt` is the one-way v3 scalar-notes → v4 note-list rewrite. No `migrate`
  stem, no dual notes type in the domain model. Other verbs refuse v3 with a
  message naming `fmt`.
- One new stem, `edit ID`: scalar replace, list append, list removal, note
  reordering, and queue position. Archive rows accept only note / evidence /
  link operations; `id` and `completed` are never editable.
- `park ID` demotes queue → horizon. `add --horizon` remains how an unfleshed
  row is created. `--notes` dies in favour of repeatable `-n/--note`.

## Why this plan exists

ctl-core owns the reusable short-option contract, and its consumers pin a
released version rather than a path dependency. qctl was nevertheless edited
across 84 files before ctl-core had a coherent, tested, released contract. The
recovery is to finish ctl-core as its own patch slice, wait for the release, and
then land qctl once.

## Evidence, 2026-08-26 (read-only)

**ctl-core** sits on `feat/require-flag-shorts` with one modified file
(`src/surface.rs`) and an untracked `.changeset/require-flag-shorts.md`. The
diff adds `Surface::long_options_without_short()` and
`Surface::require_shorts(allow)`, two walker helpers, and four unit tests.
`BTreeSet` is already imported, so the diff is plausibly compilable — nothing has
been built. The changeset is patch and its prose is already operator-facing.

**qctl** sits on `feat/030-row-edit`: 84 modified files, 4 untracked fixture
directories, `tasks.yaml` untouched. `mise run verify` fails. Format and clippy
pass; `tests/cli.rs` reports 36 passed / 2 failed and **cargo aborts there**, so
`mutations.rs`, `graph.rs`, `ledger_shape.rs`, `rows_*.rs`, `schema_file.rs`,
`usage.rs`, and `verbs.rs` never ran. Their state is unknown, not green.

**Both failures share one root cause.** `edit` (through `revise_fields`) and the
`fmt` v3 rewrite (through `replace_row_value`) each write a list by handing
`Op::Replace(Value::Sequence(..))` to yamlpatch. In yamlpatch 1.29.0,
`apply_value_replacement` takes the key-value branch and emits
`format!("{key_part} {val_str}")`, where a sequence renders as `- a\n- b`. The
result is `notes: - a` followed by an unindented `- b` — not valid YAML, which is
the identical `input is not valid YAML: error at line 1, column 1` in both tests.
`src/document.rs`'s `rewrite_list` already carries the comment that `Op::Replace`
cannot write a non-empty sequence back into a mapping. The constraint was
documented in one place and violated in two others.

**`tasks.yaml` was never dogfooded.** QCTL-030's acceptance still names
`qctl set`, and its `notes` is still a v3 scalar.

## Phase 1 — ctl-core, its own patch slice

The guard belongs on `Surface`, from the code rather than from preference:
`Surface` already extracts `SurfaceArgument { short, long, hidden, … }`
recursively from a Clap type, and ctl-core's own guidance names `Surface` as the
seam consumers build in tests to byte-compare committed operator documents. The
reusable contract is therefore a query (`long_options_without_short`) plus an
assertion over it (`require_shorts(allow)`) on a type consumers already build.
The `allow` list exists because `FormatLong` and `ColorLong` deliberately leave
`-f` and `-c` to the consuming CLI — a chassis fact, not a loophole.

Work: review the uncommitted diff for correctness — in particular that hidden
arguments are included and inherited globals are not double-counted on children —
confirm the tests cover the allow list and the `OutputArgs` default, keep the
changeset at patch, commit with a Conventional Commit, push, open a PR.

Accept: ctl-core `mise run verify` green. `require_shorts([])` fails naming every
offending command path and long option. `require_shorts` with the `format` /
`color` / `no-color` allowances passes for a CLI that owns `-f` and `-c`. One
patch changeset; no hand-edited version or changelog. PR open, CI green,
**not merged**.

## Phase 2 — qctl work safe in parallel

Nothing here depends on the released ctl-core:

- Schema v4: `notes: Vec<String>` on all three row types, `VERSION = 4`,
  regenerated `schema/tasks.schema.json`.
- The `fmt` v3→v4 rewrite and the `edit` list-write mechanic, including the
  `Op::Replace` sequence fix.
- `park ID` demotion, `-n/--note`, fixture and `tasks.yaml` conversion,
  `src/instructions.md` and `skills/qctl/SKILL.md` prose, dogfooding.
- The short-option letters themselves, which are qctl's choice.

Must wait for the release: replacing the hand-rolled long-only walker in
`tests/cli.rs` with `Surface::require_shorts`, and the `Cargo.toml` change that
pins the new ctl-core and enables its `surface` feature. qctl does not enable
`surface` today (`features = ["app", "usage"]`), so this phase also decides where
the feature is turned on. A `[dev-dependencies]` entry is the right shape under
the edition-2024 resolver — test builds gain MiniJinja and Serde, a release build
does not — but that must be verified against the release binary's dependency
graph, not assumed.

## Phase 3 — qctl, one complete patch release

Runs only after Phase 1 is merged by the operator **and** released. Pin the exact
released ctl-core version, delete the local walker, adopt `require_shorts` with
its allow list, then finish the slice: schema v4, `fmt` as the one-way rewrite,
`edit ID`, `park ID`, the complete short map, instructions / skill / tests, and
dogfooding QCTL-030 through the real commands. One patch changeset, one PR.

Accept:

- `mise run verify` green with the full suite executing to the end.
- `mise run schema` produces no diff.
- Every mutation fixture has an accepted snapshot.
- `qctl fmt` on a v3 ledger produces a v4 ledger that `qctl check` accepts.
- `tasks.yaml` is v4, QCTL-030's acceptance no longer says `qctl set`, and the
  row is archived by `qctl archive` rather than by hand.
- `qctl --help` and `qctl instructions` match the shipped grammar.

## Current working-tree disposition

Keep:

- `src/schema.rs` v4 notes — exactly the contract; the `uniqueItems` extension
  matches the `acceptance` precedent.
- `src/report.rs`, `src/presentation.rs`, `src/main.rs` — small and mechanical.
- Fixture `schema_version: 3 → 4` churn (~30 files) and the matching `.snap`
  one-line changes — unavoidable, and unaffected by the list-write bug.
- `park-a-row` and `park-without-a-horizon-key` fixtures — correctly rewritten
  to the `park ID` transition.
- The 4 untracked fixture directories (`edit-a-queued-title`,
  `edit-appends-a-note`, `edit-moves-a-row-before`, `fmt-rewrites-v3-notes`) —
  inputs only. Accepting a snapshot now would bless broken output.
- `src/instructions.md` and `skills/qctl/SKILL.md` — good prose that already
  folds in the no-splice rule; re-verify against the final flag set.

Keep with one consolidation: `src/ledger.rs`'s v3 refusal is right, but
`src/check.rs` inlines a second copy of the same version probe. Both should call
the one function.

Rewrite:

- `src/document.rs::revise_fields` — duplicates `revise()` and uses the sequence
  `Replace` path that cannot work. The list write must go through a span
  replacement like `rewrite_list`, generalized to serializer-rendered items.
- `.changeset/no-splice-id.md` — the current text is a flag inventory, not
  release prose, and it displaced the no-splice note it replaced. One patch
  fragment should cover both changes in operator terms.

Keep the shape, fix the list path: `src/document.rs`'s `replace_row_value`,
`remove_row_key`, and `row_key_shape` hit the same defect from `fmt`.
`src/format.rs`'s v3 upgrade design is correct and blocked on the same bug.

Restructure while retaining intent: `src/mutate.rs`'s `edit` (+518 lines). Three
`edit_*` functions repeat one `apply_list` / `field_changes` / `rewrite_row`
sequence over different subsets, plus four `refuse_*` guards that hand-expand a
permission matrix. One row-kind descriptor carrying allowed fields, threaded
through a single code path, removes most of it. `edit_queue`'s inline blocker
re-validation also duplicates `add_queue`'s.

Re-derive: `tests/fixtures/mutations/start-in-a-real-ledger/before.yaml` (206
lines) is a real-ledger copy whose scalar notes became lists. Regenerate it from
the final `fmt` once the rewrite works rather than trusting a hand conversion.

Recovery is non-destructive. `feat/030-row-edit` has one clean committed
ancestor (`docs: forbid splicing a new ledger row`), so the whole dirty pile is
recoverable by inspection against it. Before resuming, record the untrusted
baseline as a dangling commit with `git stash create` — which does not touch the
working tree — and put that SHA in this row's notes. Then commit forward in
coherent checkpoints: schema v4 and fixtures, the document-layer list write with
its two failing tests turned green, `edit`, `park`, docs and dogfood — each green
on focused tests before the next begins.

## Verification

- Focused, while iterating: the two currently failing `tests/cli.rs` cases, then
  `tests/mutations.rs` per new fixture, then `tests/usage.rs` after the flag set
  settles.
- Full, at each slice boundary: `mise run verify` in the owning repository, run
  to completion. A suite that aborts early has not passed.
- `mise run schema` after any `src/schema.rs` change; a diff is a failure.
- Never claim success from an exit message. Inspect `git status`, the generated
  schema, help/usage output, and PR state.

## Release and merge gates

- Two patch changesets: one in ctl-core for the guard, one in qctl for this
  slice. Operator-facing prose — why it matters and what the CLI now does — not
  flag inventories or schema plumbing.
- No hand-edited versions or changelogs, ever.
- **Neither PR is merged by the agent.** Phase 3 begins only after the operator
  merges the ctl-core PR and the release publishes. Until a released version
  exists, qctl may not depend on it: a path dependency and a duplicated local
  guard are both forbidden.

## Open decision, before Phase 3

`edit` currently declares 23 short options, including unmnemonic letters for
`--front`, `--back`, `--remove-acceptance`, `--remove-link`, and
`--remove-blocked-by`. They are collision-free, but a flag nobody can recall is
not an intuitive API.

Two families could collapse instead:

- the four position flags (`--before`, `--after`, `--front`, `--back`) into one
  `-p/--position front|back|before:ID|after:ID`;
- the five removal flags into one repeatable `-x/--remove note:2`,
  `-x acceptance:'exact text'`.

That takes `edit` from 23 shorts to roughly 13, keeps notes removable and
reorderable, and serves "every parameter has a short" better by having fewer
parameters. It diverges from the drafted flag list, so it is the operator's call.
Phase 1 does not depend on the answer; Phase 3 does.

## Stop conditions

Stop and ask if:

- ctl-core `verify` fails inside `Surface` rather than in the new methods;
- enabling the `surface` feature pulls MiniJinja into the qctl release binary;
- the generalized list write cannot preserve an inline flow sequence such as
  `acceptance: [It holds.]` without rewriting rows that did not change;
- `qctl fmt` on the real `tasks.yaml` produces a diff beyond the notes rewrite
  and the version line;
- the ctl-core PR is still unmerged when Phase 2 finishes — Phase 3 waits rather
  than falling back to a path dependency or a duplicated local guard.
