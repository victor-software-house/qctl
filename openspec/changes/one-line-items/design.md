# Design

## Context

See `proposal.md` for the measured problem.
Today list items reach the file three ways:

1. Notes go through `Document::push_notes`, one `push_text` per note, in the style `notes::style` picks: `Auto`, `Folded`, or `Literal`.
2. Other items appended to a block list go through `yamled::Document::push`, which renders each through yamled's automatic order.
3. A new row (`Document::append`), a list rewritten whole (`Document::write_list` when it is not an append), and `rewrite_blockers` serialize the whole value through `serde-saphyr`, which picks its own quoting.

yamled YMD-011 adds `TextStyle::Line` (one line, ignoring the node's current style) and makes the automatic order double-quote text with an apostrophe and no double quote.

## Goals / Non-Goals

**Goals:**

1. One function writes every row list item, in every verb and in `fmt`.
2. `fmt` turns every block list item into that shape and changes no text.

**Non-Goals:**

1. Re-quoting a one-line item that is already written, such as an existing `'it''s'`. Its bytes stay; the operator's rule governs what qctl writes, and `fmt` changes only block items and notes holding a line break.
2. Top-level scalars and row scalars (`title`, `outcome`, `plan`). They keep yamled's replace rules.
3. Flow-sequence lists. A verb replaces one whole through yamled and `fmt` skips it, so neither splits its notes. No ledger in the census holds a non-empty flow list.
4. A `check` error for a note holding a line break. `fmt --check` is the shape gate.

## Decisions

### 1. One item writer on `TextStyle::Line`

`Document` gets one private writer, `push_item(path, text)`, that calls `push_text(path, text, TextStyle::Line)`, and an insert form at an index.
`push_items`, `push_notes`, `append`, `write_list`, and `rewrite_blockers` all go through it.
`notes::style` is removed.

A list rewritten whole is emptied and refilled item by item, as notes already are.
A new row is pushed with each list field set to `[]`, then each field is filled item by item, so key order stays what the row type serializes and an empty `blocked_by` stays `blocked_by: []`.

Alternative: keep `TextStyle::Auto`.
It lost because `Auto` keeps a node's block style on `replace_text`, which `fmt` needs, and turns multi-line text into a literal block; `Line` states the rule in the call.

Alternative: write the quoted text in qctl and hand yamled a raw line.
It lost because yamled owns quoting and the read-back check; a second quoting rule in qctl is the drift this change removes.

### 2. Refuse a multi-line note at the argument

`-n/--note` on `add` and `edit` gets a clap value parser that refuses a value holding `\n` or `\r`, with the message: a note is one line; pass each line as its own `-n/--note`.
The refusal happens before the ledger is read, so nothing is written.

Alternative: split the value into one note per line.
It lost because a pasted paragraph hard-wrapped at a column would become sentence fragments, one per item; the author knows where the real breaks are.
The one ledger note of that shape today (33 multi-line notes, 1 hard-wrapped) shows the risk is real.

### 3. `fmt` rewrites block items in place

A new `fmt` step, after the schema 3 upgrade and before sections move, walks each row of `queue`, `archive`, and `horizon`.
For each of `notes`, `acceptance`, `evidence`, `links`, and `blocked_by` that is a block sequence, every item whose node style is `Literal` or `Folded` is rewritten, and so is a note in any style whose split (item 1) differs from its text, such as a double-quoted note holding `\n` escapes. Items run last first, so earlier indexes stay valid:

1. A note: its text split at each `\n` or `\r`, the same breaks `-n/--note` refuses, lines that are empty or only spaces dropped. The first line replaces the item with `replace_text(…, TextStyle::Line)`; each further line is inserted after it with the insert form of the item writer. A note with no line left, one holding only spaces and line breaks, stays one item: its whole text is written with `TextStyle::Line`, escaped, so no note and no owned comment disappears. The split, with this fallback, is one function that `write_list` (§4) also uses, so a later whole-list rewrite keeps the note too.
2. Any other field: `replace_text(…, TextStyle::Line)` with the item's text as is.

Each rewritten value is read back from the parsed ledger, not from the source bytes, so the text cannot drift.
Inserts run inside `Document::tight`, so split notes are not set off by blank lines.
`replace_text` replaces only the value, so a comment the item owns stays above it.

Alternative: remove the block item and insert one-line items at its index.
It lost because a remove takes the comments the item owns.

Alternative: join a hard-wrapped literal note into paragraphs, split at blank lines.
It lost because it changes text, and no note today has a blank line to split at, so every multi-line note, list-shaped ones included, would collapse into one item. The operator chose a per-line split on 2026-10-15.

### 4. A verb rewriting notes splits the same way

`write_list` for `notes` maps each note through the same line split function as `fmt`, blank-note fallback included, before refilling the list.
On a block list of a ledger `fmt` has already run on, no note holds a line break outside the blank-note fallback, so the split is the identity.

### 5. Overrides

The new `fmt` step has no style option and no disable path.
`fmt` exists to apply one shape, and an option that keeps block items would keep two writers alive.
A ledger that must keep a block item cannot run `fmt`; none is known.
`normalize_on_write: true` already runs `fmt`'s normalization on every verb, so such ledgers get the rewrite on their next write.

## Risks / Trade-offs

1. [`fmt --check` hooks fail after consumers take the release] → Each ledger repository runs `qctl fmt` once in the same pull request that bumps qctl. The rollout list in `tasks.md` names them.
2. [A hard-wrapped literal note splits into fragments] → One known note, in a repository outside ctl. Its owner edits it into paragraphs in that repository's `fmt` pull request.
3. [An escaped `\n` in a non-note item reads worse than a block] → No such item exists today (0 of 77 block items in other fields hold a line break).
4. [yamled `Line` changes] → qctl pins the yamled patch release; the mutation snapshots fail on any change in the written shape.

## Migration Plan

1. yamled releases YMD-011 as a patch; qctl bumps it.
2. qctl ships this change in its next release. The bump level is the operator's call: patch follows the house rule, minor would mark the `fmt --check` break.
3. Each ledger repository bumps its qctl pin and runs `qctl fmt` once, in its own pull request, made by its owner.
4. Rollback: revert the qctl release pin. A single-line item `fmt` rewrote stays valid YAML and reads back as the same string. A multi-line note `fmt` split stays split: the older qctl reads it as several notes, and joining them again is a hand edit.
