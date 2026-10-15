# Design

## Context

See `proposal.md` for the measured problem.
Today list items reach the file three ways:

1. Notes go through `Document::push_notes`, one `push_text` per note, in the style `notes::style` picks: `Auto`, `Folded`, or `Literal`.
2. Other items appended to a block list go through `yamled::Document::push`, which renders each through yamled's automatic order.
3. A new row (`Document::append`), a list rewritten whole (`Document::write_list` when it is not an append), and `rewrite_blockers` serialize the whole value through `serde-saphyr`, which picks its own quoting.

yamled YMD-011 splits writing into two settings: `TextStyle` picks the layout of one write (`Line` is one line whatever the node's style), and the document's `Quotes`, set with `Document::with_quotes`, picks the quotes of every write.
`Quotes::Auto` is a documented order: plain when the text reads back, else single quotes without an apostrophe, else double quotes without a double quote, else fewer escapes, ties to single.
It also adds `Document::batch`, which applies many edits with one parse, and skips a splice that writes the bytes already there.

## Goals / Non-Goals

**Goals:**

1. A ledger declares its list-item layout and its quotes, and every writer and `fmt` honour both.
2. One function writes every row list item, in every verb and in `fmt`.
3. `fmt` brings an existing ledger into the declared shape without changing any value, with one parse for all its scalar edits.

**Non-Goals:**

1. Row key order. The operator left it out on 2026-10-15.
2. The layout of row scalars (`title`, `outcome`, `plan`). A block `outcome` stays a block; only one-line scalars are re-quoted.
3. Flow-sequence lists. A verb replaces one whole through yamled and `fmt` leaves its layout alone, so neither splits its notes. No ledger in the census holds a non-empty flow list.
4. A command-line flag or environment variable for either option. They describe the file, like `indent`; shared rule 7 covers flags.
5. A `check` error for a block item or a disagreeing quote. `check` validates values, where both forms are the same string; `fmt --check` is the shape gate.

## Decisions

### 1. Two style options, each one yamled setting

`Style` gains `list_items: ListItems` (`line`, `auto`) and `quotes: Quotes` (`auto`, `single`, `double`), each with a doc comment stating its rule, and each mapped onto the yamled type it names.
`auto` means "yamled chooses" in both, as the operator asked.
The defaults are `line` and `auto`, the operator's decision of 2026-10-15.
Unlike the earlier options, whose defaults kept what qctl already wrote, these defaults change the written shape; the `Style` doc comment says so, and each ledger's rollout `fmt` pull request absorbs the change.
`schema_version` stays 4: both fields default, so an existing ledger still validates.

Alternative: a fixed one-line shape with no option.
It lost because the operator requires every ctl behaviour to be declared and controllable, and QCTL-021 already asked for quotes in `style`.

### 2. One item writer

`Document::new` takes the ledger's style and calls `with_quotes`.
`Document` gets one private writer for list items that calls `push_text` or `insert_item_text` with the layout `list_items` maps to.
`push_notes`, `push_items`, `append`, `write_list`, and `rewrite_blockers` all go through it, and `notes::style` is removed.
A list rewritten whole is emptied and refilled item by item.
A new row is pushed with each list field empty, then each field is filled item by item, so key order stays what the row type serializes and an empty `blocked_by` stays `blocked_by: []`.

Alternative: write the quoted text in qctl and hand yamled a raw line.
It lost because yamled owns quoting and its proof; a second quoting rule in qctl is the drift this change removes.

### 3. A multi-line note under `line` is refused by the verb

Under `list_items: line`, `add` and `edit` refuse a `-n/--note` value holding `\n` or `\r` after reading the ledger's style and before writing, with the message: a note is one line; pass each line as its own `-n/--note`.
Under `auto`, the note is written as one literal block.
The check moves from the argument parser into the verb because it depends on the ledger.

Alternative: split the value into one note per line.
It lost because a pasted paragraph hard-wrapped at a column would become sentence fragments, one per item; the author knows where the real breaks are.
The one ledger note of that shape today (33 multi-line notes, 1 hard-wrapped) shows the risk is real.

### 4. `fmt` writes block items on one line under `line`

Under `list_items: line`, a `fmt` step walks each row of `queue`, `archive`, and `horizon`, record kind by record kind.
For each list field that is a block sequence, every item whose node style is `Literal` or `Folded` is rewritten, and so is a note in any style whose split differs from its text, such as a double-quoted note holding `\n` escapes:

1. A note: its text split at each `\n` or `\r`, the same breaks `-n/--note` refuses, lines that are empty or only spaces dropped. The first line replaces the item with `replace_text(…, TextStyle::Line)`; each further line is inserted after it. A note with no line left stays one item: its whole text is written with `TextStyle::Line`, escaped, so no note and no owned comment disappears. The split, with this fallback, is one function that `write_list` (§7) also uses.
2. Any other field: `replace_text(…, TextStyle::Line)` with the item's text as is.

Each value is read back from the parsed ledger, not from the source bytes, so the text cannot drift.
`replace_text` replaces only the value, so a comment the item owns stays above it.
Under `auto` the step does nothing.

Alternative: remove the block item and insert one-line items at its index.
It lost because a remove takes the comments the item owns.

Alternative: join a hard-wrapped literal note into paragraphs, split at blank lines.
It lost because it changes text, and no note today has a blank line to split at. The operator chose a per-line split on 2026-10-15.

### 5. `fmt` re-quotes one-line strings to the declared quotes

A `fmt` step visits every string scalar a ledger holds, by record kind: `prefix` and `active` at the top, and each row kind's string fields and list items.
A scalar written on one line (plain, single-, or double-quoted) is replaced with its own value through `replace_text(…, TextStyle::Line)`, which renders it in the document's quotes and ignores its current style.
yamled skips a splice that writes the bytes already there, so a scalar that already agrees costs nothing.
A block scalar keeps its layout, and keys are left alone.

Alternative: walk the parsed document generically.
It lost because a schema per record kind states which fields are strings, and a generic walk would also visit keys and non-string values.

### 6. One batch for the scalar steps

Steps 4 and 5 run inside one `Document::batch`, so all their edits cost one parse.
They touch separate bytes: step 4 edits block items and split notes, and step 5 one-line scalars, which step 4 never names.
The structural steps that follow (archive order, indent, section order) need the edited index, so each runs after the batch; a step that changes nothing costs no parse.
A ledger `fmt --check` already accepts parses once.

### 7. A verb rewriting notes splits the same way

Under `line`, `write_list` for `notes` maps each note through the same split function as `fmt`, blank-note fallback included, before refilling the list.
On a block list `fmt` has already run on, no note holds a line break outside the fallback, so the split is the identity.

### 8. Overrides

`list_items: auto` keeps block items and accepts multi-line notes; `quotes: single` or `double` pins one quote style.
`normalize_on_write: true` already runs `fmt`'s normalization on every verb, so such ledgers get both steps on their next write.

## Risks / Trade-offs

1. [`fmt --check` hooks fail after consumers take the release] → Each ledger repository runs `qctl fmt` once in the same pull request that bumps qctl. The rollout list in `tasks.md` names them, with each ledger's measured change.
2. [The re-quote step widens the rollout diff] → It touches only strings whose quotes disagree; the count per public ledger is measured with the built release before rollout.
3. [A hard-wrapped literal note splits into fragments] → One known note, in a repository outside ctl. Its owner edits it into paragraphs in that repository's `fmt` pull request.
4. [An escaped `\n` in a non-note item reads worse than a block] → No such item exists today (0 of 77 block items in other fields hold a line break).
5. [yamled rendering changes] → qctl pins the yamled patch release; the mutation snapshots fail on any change in the written shape.

## Migration Plan

1. yamled releases YMD-011 as a patch; qctl bumps it, and ctl-core moves to the same yamled release so the graph carries one copy.
2. qctl ships this change as a patch, per the house rule.
3. Each ledger repository bumps its qctl pin and runs `qctl fmt` once, in its own pull request, made by its owner.
4. Rollback: revert the qctl release pin. A line `fmt` rewrote stays valid YAML and reads back as the same string. A multi-line note `fmt` split stays split: the older qctl reads it as several notes, and joining them again is a hand edit. An older qctl refuses the new `style` keys, so a ledger that declares them must drop them first.
