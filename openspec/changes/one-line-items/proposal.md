# Write every list item on one line

## Why

qctl writes a one-line note that needs quoting as a folded block, because `src/document/notes.rs` picks `TextStyle::Folded` for it.
Nearly every note starts with a date and a colon, so nearly every note takes that path.
A note given with line breaks becomes a literal block.
Acceptance, evidence, links, and blockers go through a second writer: a new row and a rewritten list are serialized whole, so their quoting is not the one notes get.

Measured on 2026-10-15 at the head of 15 qctl ledgers, 6 public and 9 private (14 on schema 4):

1. Block-scalar list items: 514 notes (`>2-`, `>-`, and `|2-`), 43 acceptance items, and 34 evidence items. The public ctl ledgers hold 81 of the block notes: verctl 33, qctl 28, forkctl 12, ctl-core 7, yamled 1.
2. Notes holding a line break: 33, all literal blocks, in two private ledgers. 32 already hold one paragraph or list item per line; one is hard-wrapped near 75 columns. None holds a blank line.
3. Acceptance and evidence items holding a line break: 0. Each block item there is one folded string.
4. Doubled single quotes (`''`): 9, of which verctl holds 5 and qctl 1.

The operator decided on 2026-10-15 that every list item has one writer and sits on one line: plain when YAML allows it, otherwise quoted, preferring double quotes when the text holds an apostrophe and no double quote.

Queue row: QCTL-039.

## What changes

1. Every verb writes every row list item (`notes`, `acceptance`, `evidence`, `links`, `blocked_by`) through one writer: one line, plain when it reads back as itself, otherwise quoted.
2. Text with an apostrophe and no double quote is double-quoted, so `''` is never written. This comes from yamled's automatic order (yamled YMD-011).
3. `add` and `edit` refuse a note that holds a line break, and say to pass each line as its own `-n/--note`.
4. `qctl fmt` rewrites each block-scalar list item as one line with the same text. A note holding line breaks becomes one item per line, in order, and an empty line is dropped. A folded item is already one string and becomes one line.
5. A verb that rewrites a row's whole `notes` list writes a note that still holds line breaks the same way, one item per line.
6. `qctl instructions` says to write a note's date followed by a comma, never a colon, and never to rewrite existing note text for it.
7. **BREAKING** for `qctl fmt --check`: a ledger with block list items fails it until `qctl fmt` runs once. Each ledger repository runs `qctl fmt` in its own pull request after the release.

## Capabilities

### New Capabilities

1. `list-items`: how qctl writes, refuses, and reformats the items of a row's list fields.

### Modified Capabilities

None. `ledger-style` keeps its requirements; its spacing rules apply to the one-line items as they do today.

## Impact

1. Code: `src/document.rs`, `src/document/notes.rs`, `src/document/rows.rs`, `src/format.rs`, `src/cli.rs`.
2. Dependency: yamled moves to the release that carries YMD-011 (`TextStyle::Line` and the apostrophe rule).
3. Contracts: `src/instructions.md`, `skills/qctl/SKILL.md`, `AGENTS.md`, the operator docs rendered from `.ctl/operator/`.
4. Tests: mutation scenarios under `tests/fixtures/mutations/` and `tests/notes.rs`.
5. Consumers: each ledger repository's first `qctl fmt` after the release rewrites its block items; the hard-wrapped note needs a hand edit.
