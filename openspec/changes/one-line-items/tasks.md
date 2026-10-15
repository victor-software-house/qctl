# Tasks

## 1. Dependency

- [ ] 1.1 Bump yamled to the release that carries YMD-011, together with ctl-core's move to it. Verify `Cargo.lock` names one yamled and `mise run verify` passes on the build host.

## 2. Style options

- [ ] 2.1 Add `list_items` and `quotes` to `Style`, with doc comments stating each rule, and regenerate the schema. Verify the "A ledger declares its list-item layout" and "A ledger declares its quotes" scenarios, including the refused values, pass on the build host.
- [ ] 2.2 Build `Document` with the ledger's quotes. Verify the "Single quotes when the ledger asks" scenario passes.

## 3. One writer

- [ ] 3.1 Route `push_items`, `push_notes`, `append`, `write_list`, and `rewrite_blockers` through one item writer in the declared layout, and remove `notes::style`. Verify the mutation scenarios for "A dated note is quoted on one line", "A note with an apostrophe is double-quoted", "A note with a comma after its date stays plain", "Acceptance uses the same writer as notes", and "A rewritten list keeps the one-line shape" pass on the build host.
- [ ] 3.2 Refuse a `-n/--note` value holding a line break on `add` and `edit` under `line`, and write it as one block under `auto`. Verify a `tests/notes.rs` test sees a non-zero exit, the error text, and an unchanged file under `line`, and the block under `auto`.
- [ ] 3.3 Split a multi-line note per line when a verb rewrites the whole `notes` list under `line`. Verify the "Removing a note splits a literal neighbour" mutation scenario passes.

## 4. fmt

- [ ] 4.1 Add the `fmt` step that rewrites block list items in place under `line`. Verify mutation scenarios for a folded dated note, a hard-wrapped folded acceptance item, a literal note split per line, a quoted note holding a line break, a note of only blank lines, a neighbour that keeps its bytes, a comment that stays above a rewritten item, `auto` keeping block items, and `fmt --check` on a block note, each on a real written file.
- [ ] 4.2 Add the re-quote step. Verify mutation scenarios for a doubled quote, a needlessly quoted stamp, and a ledger that already agrees.
- [ ] 4.3 Run both steps in one `Document::batch`. Verify on the build host, on the largest consumer ledger, the wall time of `qctl fmt --check` and a full `qctl fmt` before and after, and record the numbers in the pull request.
- [ ] 4.4 Prove `fmt` is idempotent and keeps every value. Verify a test that runs `qctl fmt` twice on a ledger with every block shape and mixed quotes, asserts the second run changes nothing, and compares each row's values before and after, notes compared after the per-line split.

## 5. Contracts

- [ ] 5.1 Update `src/instructions.md`: the two options, notes one line each under `line`, the date-comma rule, and what `fmt` rewrites. Verify `qctl instructions` output carries them and `UPDATE_OPERATOR_DOCS=1 cargo test operator_docs` leaves the committed renders matching.
- [ ] 5.2 Update `skills/qctl/SKILL.md`, the `-n/--note` help in `src/cli.rs`, and the `notes` and style bullets in `AGENTS.md`. Verify `git grep -n "literal block\|folded block"` finds no stale note-style guidance.
- [ ] 5.3 Close QCTL-021 as absorbed by this change, with evidence naming the pull request. Verify `qctl check` passes and QCTL-021 sits in the archive.
- [ ] 5.4 Add a patch changeset. Verify `mise run verify` passes on the build host and the pull request's CI is green.

## 6. Rollout after the release

Each repository's owner bumps its qctl pin and runs `qctl fmt` once, in its own pull request.
Block items per public repository, as counted on 2026-10-15; the re-quoted strings per ledger are counted with the release before rollout:

- [ ] 6.1 verctl: 33 notes, 5 doubled quotes.
- [ ] 6.2 qctl: 28 notes, 1 acceptance item, 3 evidence items, 1 doubled quote.
- [ ] 6.3 forkctl: 12 notes.
- [ ] 6.4 ctl-core: 7 notes.
- [ ] 6.5 yamled: 1 note, plus the notes YMD-011 added.

unixd has no block items.
Nine private ledgers hold the rest; their owners get the list outside this repository.
One of them holds the hard-wrapped multi-line note, which its owner edits into paragraphs before running `qctl fmt`.
One is still on schema 3; its first `qctl fmt` upgrades it and writes its notes one per line in the same run.
