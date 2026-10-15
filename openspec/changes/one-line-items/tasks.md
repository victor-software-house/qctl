# Tasks

## 1. Dependency

- [ ] 1.1 Bump yamled to the release that carries YMD-011. Verify `Cargo.lock` names it and `mise run verify` passes on the build host.

## 2. One writer

- [ ] 2.1 Route `push_items`, `push_notes`, `append`, `write_list`, and `rewrite_blockers` through one item writer on `TextStyle::Line`, and remove `notes::style`. Verify the mutation scenarios for "A dated note is quoted on one line", "A note with an apostrophe is double-quoted", "A note with a comma after its date stays plain", "Acceptance uses the same writer as notes", and "A rewritten list keeps the one-line shape" pass on the build host.
- [ ] 2.2 Refuse a `-n/--note` value holding a line break on `add` and `edit`. Verify a `tests/notes.rs` test that runs the binary with a two-line note sees a non-zero exit, the error text, and an unchanged file.
- [ ] 2.3 Split a multi-line note per line when a verb rewrites the whole `notes` list. Verify the "Removing a note splits a literal neighbour" mutation scenario passes.

## 3. fmt

- [ ] 3.1 Add the `fmt` step that rewrites block list items in place. Verify mutation scenarios for a folded dated note, a hard-wrapped folded acceptance item, a literal note split per line, a neighbour that keeps its bytes, a comment that stays above a rewritten item, and `fmt --check` on a block note, each on a real written file.
- [ ] 3.2 Prove `fmt` is idempotent and keeps every value. Verify a test that runs `qctl fmt` twice on a ledger with every block shape, asserts the second run changes nothing, and compares `qctl show --format json` for each row before and after, notes compared after the per-line split.

## 4. Contracts

- [ ] 4.1 Update `src/instructions.md`: notes are one line each, the date-comma rule, and `fmt` rewriting block items. Verify `qctl instructions` output carries both rules and `UPDATE_OPERATOR_DOCS=1 cargo test operator_docs` leaves the committed renders matching.
- [ ] 4.2 Update `skills/qctl/SKILL.md`, the `-n/--note` help in `src/cli.rs`, and the `notes` bullet in `AGENTS.md`. Verify `git grep -n "literal block\|folded block\|>-\||-"` finds no stale note-style guidance.
- [ ] 4.3 Add a changeset. Verify `mise run verify` passes on the build host and the pull request's CI is green.

## 5. Rollout after the release

Each repository's owner bumps its qctl pin and runs `qctl fmt` once, in its own pull request.
Block items per public repository, as counted on 2026-10-15:

- [ ] 5.1 verctl: 33 notes.
- [ ] 5.2 qctl: 28 notes, 1 acceptance item, 3 evidence items.
- [ ] 5.3 forkctl: 12 notes.
- [ ] 5.4 ctl-core: 7 notes.
- [ ] 5.5 yamled: 1 note, plus the notes YMD-011 added.

unixd has no block items.
Nine private ledgers hold the rest; their owners get the list outside this repository.
One of them holds the hard-wrapped multi-line note, which its owner edits into paragraphs before running `qctl fmt`.
One is still on schema 3; its first `qctl fmt` upgrades it and writes its notes one per line in the same run.
