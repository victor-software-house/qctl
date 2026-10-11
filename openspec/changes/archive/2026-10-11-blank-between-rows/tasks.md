# Tasks

- [x] 1. `Style::row_spacing` with `as_written` (default) and `blank_line`, in
  the generated schema. Proof: `the_committed_schema_is_what_the_types_say`
  passes in `mise run verify` on the build host.
- [x] 2. `fmt` inserts a missing separator, keeps the first row tight to its
  key, puts the blank line above a row's comments, and skips a row ending in a
  kept block scalar. Proof: new `fmt-*` mutation scenarios pass in
  `mise run verify` on the build host.
- [x] 3. Instructions and the operator skill describe the option and no longer
  say `fmt` never inserts a blank line. Proof: `operator_docs` tests pass in
  `mise run verify` on the build host.
- [x] 4. A minor changeset (a new style option, per AGENTS.md). Proof:
  `.changeset/blank-between-rows.md`.

## Evidence

2026-10-11 UTC. `mise run verify` passes on a build runner. The four new
mutation scenarios record:

1. `fmt-separates-rows-when-the-style-says-so`: one blank line inserted
   between two adjacent queue rows, a doubled archive separator collapsed to
   one, and none added under `queue:`.
2. `fmt-leaves-row-spacing-as-written`: the same ledger without the option
   keeps its adjacent rows; only the doubled blank line collapses.
3. `fmt-puts-the-separator-above-a-comment`: the blank line lands above
   `# waiting on review`, which stays on `DEMO-002`.
4. `fmt-leaves-a-kept-block-row-tight`: a row ending in a `|+` note is left
   unchanged.

`fmt_check_refuses_a_lost_row_separator` shows `fmt --check` refusing the gap
without writing, and `refuses_a_ledger::case_*_a_row_spacing_qctl_does_not_know`
shows `check` naming `row_spacing` for an unknown value. The operator skill
never said `fmt` refuses blank lines, so only the instructions changed.
