# Write list items and quotes as the ledger declares

## Why

qctl writes a one-line note that needs quoting as a folded block, because `src/document/notes.rs` picks `TextStyle::Folded` for it.
Nearly every note starts with a date and a colon, so nearly every note takes that path.
A note given with line breaks becomes a literal block.
Acceptance, evidence, links, and blockers go through a second writer: a new row and a rewritten list are serialized whole, so their quoting is not the one notes get.
No ledger can say how its items are laid out or how its strings are quoted, so one file writes one fact several ways (QCTL-021).

Measured on 2026-10-15 at the head of 15 qctl ledgers, 6 public and 9 private (14 on schema 4):

1. Block-scalar list items: 514 notes (`>2-`, `>-`, and `|2-`), 43 acceptance items, and 34 evidence items. The public ctl ledgers hold 81 of the block notes: verctl 33, qctl 28, forkctl 12, ctl-core 7, yamled 1.
2. Notes holding a line break: 33, all literal blocks, in two private ledgers. 32 already hold one paragraph or list item per line; one is hard-wrapped near 75 columns. None holds a blank line.
3. Acceptance and evidence items holding a line break: 0. Each block item there is one folded string.
4. Doubled single quotes (`''`): 9, of which verctl holds 5 and qctl 1.

The operator decided on 2026-10-15 that every list item has one writer and sits on one line by default, that layout and quoting are each a declared, documented ledger setting rather than a fixed rule, that `auto` names "let yamled choose" in both, and that QCTL-021 folds into this change while row key order stays out.

Queue row: QCTL-039, which absorbs QCTL-021.

## What changes

1. The ledger's `style` block gains two options, each mapping to one yamled setting (yamled YMD-011):
   1. `list_items: line | auto`, default `line`: the layout of every row list item, yamled `TextStyle::Line` or `TextStyle::Auto`.
   2. `quotes: auto | single | double`, default `auto`: the quotes of every string qctl writes, yamled `Quotes`. Under `auto`, text is plain when it reads back, else single-quoted without an apostrophe, else double-quoted without a double quote, else whichever needs fewer escapes, so `''` is never written for an apostrophe.
2. Every verb writes every row list item (`notes`, `acceptance`, `evidence`, `links`, `blocked_by`) through one writer, in the declared layout and quotes.
3. Under `list_items: line`, `add` and `edit` refuse a note that holds a line break, and say to pass each line as its own `-n/--note`. Under `auto`, such a note is written as one literal block.
4. Under `list_items: line`, `qctl fmt` rewrites each block-scalar list item as one line with the same text. A note holding line breaks becomes one item per line, in order, and an empty line is dropped.
5. `qctl fmt` re-quotes every one-line string scalar whose quoting disagrees with `quotes`, so a mix cannot survive the `fmt --check` hook.
6. `qctl fmt` applies its scalar edits in one yamled batch, parsed once.
7. A verb that rewrites a row's whole `notes` list under `line` writes a note that still holds line breaks one item per line.
8. `qctl instructions` says to write a note's date followed by a comma, never a colon, and never to rewrite existing note text for it.
9. **BREAKING** for `qctl fmt --check`: a ledger with block list items, or with strings quoted against its declared `quotes`, fails it until `qctl fmt` runs once. Each ledger repository runs `qctl fmt` in its own pull request after the release.

## Capabilities

### New Capabilities

1. `list-items`: how qctl writes, refuses, and reformats the items of a row's list fields.

### Modified Capabilities

1. `ledger-style`: the `list_items` and `quotes` options, `fmt` re-quoting to the declared quotes, and the scenario for "A kept block scalar is never extended", which moves from a `|+` note to a `|+` outcome.

## Impact

1. Code: `src/schema.rs`, `src/document.rs`, `src/document/notes.rs`, `src/document/rows.rs`, `src/format.rs`, `src/mutate.rs`, `src/cli.rs`.
2. Dependency: yamled moves to the release that carries YMD-011 (`Quotes`, `TextStyle::Line`, `Document::batch`).
3. Schema: the generated JSON schema gains the two `style` options; `schema_version` stays 4, because both default.
4. Contracts: `src/instructions.md`, `skills/qctl/SKILL.md`, `AGENTS.md`, the operator docs rendered from `.ctl/operator/`.
5. Tests: mutation scenarios under `tests/fixtures/mutations/` and `tests/notes.rs`.
6. Consumers: each ledger repository's first `qctl fmt` after the release rewrites its block items and re-quotes strings; the hard-wrapped note needs a hand edit.
