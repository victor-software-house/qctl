# Tasks

The build host for every check below is the pull request's CI ([run on 3a8a5a5][ci]); renders and measurements ran on a Linux Namespace instance at the commit each note names.

## 1. Dependency

- [x] 1.1 Bump yamled to the release that carries YMD-011, together with ctl-core's move to it. Verify `Cargo.lock` names one yamled and `mise run verify` passes on the build host.

  2026-10-17 UTC: [6dcb637][6dcb637] takes ctl-core 0.6.24 and yamled 0.0.8; `Cargo.lock` has one `name = "yamled"` entry, and Verify passed in [CI][ci].

## 2. Style options

- [x] 2.1 Add `list_items` and `quotes` to `Style`, with doc comments stating each rule, and regenerate the schema. Verify the "A ledger declares its list-item layout" and "A ledger declares its quotes" scenarios, including the refused values, pass on the build host.

  2026-10-17 UTC: `tests/ledger_shape.rs` refuses `list_items: folded` and `quotes: backtick` and names the field; `schema/tasks.schema.json` was regenerated on the render instance. Both pass in [CI][ci].
- [x] 2.2 Build `Document` with the ledger's quotes. Verify the "Single quotes when the ledger asks" scenario passes.

  2026-10-17 UTC: `tests/notes.rs` `single_quotes_when_the_ledger_asks` passes in [CI][ci].

## 3. One writer

- [x] 3.1 Route `push_items`, `push_notes`, `append`, `write_list`, and `rewrite_blockers` through one item writer in the declared layout, and remove `notes::style`. Verify the mutation scenarios for "A dated note is quoted on one line", "A note with an apostrophe is double-quoted", "A note with a comma after its date stays plain", "Acceptance uses the same writer as notes", and "A rewritten list keeps the one-line shape" pass on the build host.

  2026-10-17 UTC: `push_items` is the one writer and `notes::style` is gone. The mutation scenarios `add-writes-each-item-on-one-line`, `edit-appends-an-acceptance-with-an-apostrophe`, `promote-writes-items-on-one-line`, and `archive-clears-a-blocker-from-a-block-list` record the shapes, and pass in [CI][ci].
- [x] 3.2 Refuse a `-n/--note` value holding a line break on `add` and `edit` under `line`, and write it as one block under `auto`. Verify a `tests/notes.rs` test sees a non-zero exit, the error text, and an unchanged file under `line`, and the block under `auto`.

  2026-10-17 UTC: `a_note_holding_a_line_break_is_refused` and `the_automatic_layout_keeps_line_breaks` pass in [CI][ci].
- [x] 3.3 Split a multi-line note per line when a verb rewrites the whole `notes` list under `line`. Verify the "Removing a note splits a literal neighbour" mutation scenario passes.

  2026-10-17 UTC: `edit-splits-a-literal-note-beside-a-removed-one` passes in [CI][ci].

## 4. fmt

- [x] 4.1 Add the `fmt` step that rewrites block list items in place under `line`. Verify mutation scenarios for a folded dated note, a hard-wrapped folded acceptance item, a literal note split per line, a quoted note holding a line break, a note of only blank lines, a neighbour that keeps its bytes, a comment that stays above a rewritten item, `auto` keeping block items, and `fmt --check` on a block note, each on a real written file.

  2026-10-17 UTC: `fmt-writes-block-items-on-one-line`, `fmt-splits-a-quoted-note-and-keeps-a-blank-one`, and `fmt-keeps-block-items-under-auto` cover these shapes, and `tests/notes.rs` covers `fmt --check`. All pass in [CI][ci]. The last two snapshots were rendered on yamled 0.0.8, in [5657c4c][5657c4c].
- [x] 4.2 Add the re-quote step. Verify mutation scenarios for a doubled quote, a needlessly quoted stamp, and a ledger that already agrees.

  2026-10-17 UTC: `fmt-requotes-strings` covers a doubled quote, a quoted stamp, a quoted `disposition` and `kind`, and an archived row with no `disposition` ([3a8a5a5][3a8a5a5]); `fmt-changes-nothing-in-a-tidy-ledger` covers a ledger that agrees. Both pass in [CI][ci].
- [x] 4.3 Run both steps in one `Document::batch`. Verify on the build host, on the largest consumer ledger, the wall time of `qctl fmt --check` and a full `qctl fmt` before and after, and record the numbers in the pull request.

  2026-10-17 UTC, on a Linux 8x16 instance at 3a8a5a5 against `main` at d4be6bc, release builds, five runs each, on a 600459-byte consumer ledger. Median wall times:

  | Command | Before | After |
  |:--|--:|--:|
  | `qctl fmt --check` | 0.018 s | 0.023 s |
  | `qctl fmt` | 0.018 s | 0.023 s |

  The first `fmt` removed 789 lines and added 497. The numbers are in the [pull request][pr].
- [x] 4.4 Prove `fmt` is idempotent and keeps every value. Verify a test that runs `qctl fmt` twice on a ledger with every block shape and mixed quotes, asserts the second run changes nothing, and compares each row's values before and after, notes compared after the per-line split.

  2026-10-17 UTC: `fmt_keeps_every_value_and_settles` passes in [CI][ci]. The same check ran on two real ledgers at 3a8a5a5, comparing every value read back as JSON with notes split per line:

  - the 600459-byte consumer ledger: values equal, notes 314 → 443 items, second `fmt --check` exits 0;
  - this repository's ledger: values equal, notes 43 → 43 items, second `fmt --check` exits 0.

  `qctl check` reported the same problems before and after on both ledgers. Those problems come from files the instance did not have, such as plan paths.

## 5. Contracts

- [x] 5.1 Update `src/instructions.md`: the two options, notes one line each under `line`, the date-comma rule, and what `fmt` rewrites. Verify `qctl instructions` output carries them and `UPDATE_OPERATOR_DOCS=1 cargo test operator_docs` leaves the committed renders matching.

  2026-10-17 UTC: the render ran with `UPDATE_OPERATOR_DOCS=1` and changed no committed render; the operator-doc comparison passes in [CI][ci].
- [x] 5.2 Update `skills/qctl/SKILL.md`, the `-n/--note` help in `src/cli.rs`, and the `notes` and style bullets in `AGENTS.md`. Verify `git grep -n "literal block\|folded block"` finds no stale note-style guidance.

  2026-10-17 UTC: outside `openspec/`, the ledger, and test fixtures, the remaining matches describe `list_items: auto`, in `src/instructions.md`, `src/schema.rs`, the generated schema, and `tests/notes.rs`.
- [x] 5.3 Close QCTL-021 as absorbed by this change, with evidence naming the pull request. Verify `qctl check` passes and QCTL-021 sits in the archive.

  2026-10-17 UTC: [ed326ae][ed326ae] archives QCTL-039 and QCTL-021, the latter with "Absorbed by QCTL-039"; `qctl check --no-git` exited 0 on the render instance.
- [x] 5.4 Add a patch changeset. Verify `mise run verify` passes on the build host and the pull request's CI is green.

  2026-10-17 UTC: `.changeset/one-line-items.md` is a patch. Verify, Template, and Kody pass in [CI][ci]. cargo-semver-checks asks for a major release for the new `Style` fields, `Change::Items`, and the new arity of `Document::new` and `Document::append`. Releases stay patch-only, so [378c973][378c973] allows exactly those failures while 0.5.8 is the newest release, and QCTL-050 removes that window.

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

[ci]: https://github.com/victor-software-house/qctl/actions/runs/39071058637
[pr]: https://github.com/victor-software-house/qctl/pull/118
[6dcb637]: https://github.com/victor-software-house/qctl/commit/6dcb637d7e611f8c3d7fe8c75158821116dbdfad
[5657c4c]: https://github.com/victor-software-house/qctl/commit/5657c4cd8e8436f2370b1a685e01e4f1ad765e90
[ed326ae]: https://github.com/victor-software-house/qctl/commit/ed326ae1f08a59a693ebc67071839d3f915e45e6
[378c973]: https://github.com/victor-software-house/qctl/commit/378c973f94cef309fe97ddd3ad12a05eb1bf4741
[3a8a5a5]: https://github.com/victor-software-house/qctl/commit/3a8a5a59845c59dbf152f5592b632c42952adb7d
