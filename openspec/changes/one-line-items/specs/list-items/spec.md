# Spec Delta

## Purpose

How qctl writes the items of a row's list fields (`notes`, `acceptance`, `evidence`, `links`, `blocked_by`): one writer, in the layout the ledger declares with `style.list_items` (one line per item by default) and the quotes it declares with `style.quotes`, and a `fmt` rewrite that brings older block items into that shape without changing their text.

## ADDED Requirements

### Requirement: Every list item is written on one line

Under `list_items: line`, the default, every verb that writes an item of a row's list field SHALL write it on one line, as the item's own line of the block list, and never as a folded or literal block.
The item SHALL be plain when plain text reads back as the same string, and quoted as the ledger's `quotes` say otherwise.
Under `list_items: auto`, the item SHALL be written as yamled's automatic layout writes it: on one line, or as a literal block when its text holds a line break.

#### Scenario: A dated note is quoted on one line

- **WHEN** `qctl add -t T -s S -o O -a A -n "2026-10-15: found in review"` runs on a `tasks.yaml` that declares `prefix: DEMO`
- **THEN** the new row's `notes` holds the line `- '2026-10-15: found in review'`
- **AND** no line of the row starts with `- >` or `- |`

#### Scenario: A note with a comma after its date stays plain

- **WHEN** `qctl edit DEMO-001 -n "2026-10-15, found in review"` runs
- **THEN** the appended line reads `- 2026-10-15, found in review`

#### Scenario: Acceptance uses the same writer as notes

- **WHEN** `qctl add -t T -s S -o O -a "Done: every row passes" -n "Done: every row passes"` runs
- **THEN** the acceptance item and the note item are written as the same text, `'Done: every row passes'`

#### Scenario: A rewritten list keeps the one-line shape

- **WHEN** `DEMO-001` has the acceptance items `First` and `Second`
- **AND** `qctl edit DEMO-001 -x acceptance:1 -a "It's checked: yes"` runs
- **THEN** the acceptance list reads `- Second` then `- "It's checked: yes"`

### Requirement: A quoted item follows the declared quotes

A list item that cannot be plain SHALL be quoted as the ledger's `quotes` option says.
Under `quotes: auto`, the default, an item whose text holds an apostrophe and no double quote SHALL be double-quoted, so `''` is never written for an apostrophe.

#### Scenario: A note with an apostrophe is double-quoted

- **WHEN** `qctl edit DEMO-001 -n "2026-10-15: it's blocked on review"` runs
- **THEN** the appended line reads `- "2026-10-15: it's blocked on review"`
- **AND** the file holds no `''`

### Requirement: An append keeps the bytes of the items before it

A verb that only appends to a list SHALL keep the bytes of every item already in it, unless the ledger declares `normalize_on_write: true`, which runs the `fmt` rewrite on every write.
A verb that rewrites a block list whole, by a removal, a reorder, or `-R`, SHALL write every item of that list in the declared layout.

#### Scenario: Appending a note leaves a folded note alone

- **WHEN** `DEMO-001`'s notes hold a `- >2-` block and the ledger does not set `normalize_on_write`
- **AND** `qctl edit DEMO-001 -n "Next."` runs
- **THEN** the `- >2-` block is unchanged and `- Next.` follows it

### Requirement: A note holding a line break is refused under the line layout

Under `list_items: line`, `qctl add` and `qctl edit` SHALL refuse a `-n/--note` value that holds a line break, before they write anything.
The error SHALL say that a note is one line and that each line is passed as its own `-n/--note`.
Under `list_items: auto`, such a note SHALL be written as one literal block.

#### Scenario: A two-line note is refused

- **WHEN** `qctl add -t T -s S -o O -a A -n <note>` runs, where `<note>` is `First line.`, a line break, and `Second line.`
- **THEN** it exits non-zero, `tasks.yaml` is unchanged, and the error names `-n/--note`

#### Scenario: The automatic layout keeps a two-line note as one block

- **WHEN** a `tasks.yaml` declares `style: {list_items: auto}`
- **AND** `qctl add -t T -s S -o O -a A -n <note>` runs, where `<note>` is `First line.`, a line break, and `Second line.`
- **THEN** the row's `notes` holds one `- |-` item whose lines are `First line.` and `Second line.`

#### Scenario: Two notes are accepted

- **WHEN** `qctl add -t T -s S -o O -a A -n "First line." -n "Second line."` runs
- **THEN** the row's `notes` holds `- First line.` then `- Second line.`

### Requirement: fmt writes block list items on one line

Under `list_items: line`, `qctl fmt` SHALL write each folded or literal item of a block list field on one line, reading as the same text.
A non-note item holding a line break SHALL stay one item, with `\n` escapes.
A rewritten item's comments SHALL stay above it.
`qctl fmt --check` SHALL fail on such an item and write nothing, and a second `qctl fmt` SHALL change nothing.

#### Scenario: A folded dated note becomes one line

- **WHEN** `DEMO-001`'s notes hold `- >2-` followed by the line `2026-09-28: the old shape`
- **AND** `qctl fmt` runs
- **THEN** the note reads `- '2026-09-28: the old shape'`
- **AND** `qctl show DEMO-001 --format json` reports the same note text as before

#### Scenario: A hard-wrapped folded acceptance item becomes one line

- **WHEN** `DEMO-001`'s acceptance holds `- >-` followed by the lines `The first half,` and `and the second half.`
- **AND** `qctl fmt` runs
- **THEN** the item reads `- The first half, and the second half.`

#### Scenario: Neighbouring items keep their bytes

- **WHEN** `DEMO-001`'s notes hold `- 'kept: as written'` followed by a `- >-` block
- **AND** `qctl fmt` runs
- **THEN** the line `- 'kept: as written'` is unchanged

#### Scenario: check mode names a block item

- **WHEN** a row holds a `- >2-` note
- **AND** `qctl fmt --check` runs
- **THEN** it exits non-zero and `tasks.yaml` is unchanged

### Requirement: fmt keeps item layout under the automatic layout

Under `list_items: auto`, `qctl fmt` SHALL leave the layout of every list item as written.

#### Scenario: The automatic layout keeps block items

- **WHEN** a `tasks.yaml` declares `style: {list_items: auto}` and a row holds a `- >2-` note
- **AND** `qctl fmt` runs
- **THEN** the `- >2-` note is unchanged

### Requirement: fmt writes a multi-line note one item per line

Under `list_items: line`, `qctl fmt` SHALL write a note of a block list whose text holds line breaks, `\n` or `\r`, in any style, as one item per line, in order, in its place.
It SHALL drop a line that is empty or holds only spaces, and with it the trailing line breaks a kept block (`|+`) carried.
A note with no other line SHALL stay one item holding its whole text, double-quoted with `\n` escapes.

#### Scenario: A literal note becomes one item per line

- **WHEN** `DEMO-001`'s notes hold `- |-` followed by the lines `Lead: the design.`, `1. First.`, and `2. Second.`
- **AND** `qctl fmt` runs
- **THEN** the notes hold `- 'Lead: the design.'`, then `- 1. First.`, then `- 2. Second.`, in the place the block held

#### Scenario: A quoted note holding a line break is split

- **WHEN** `DEMO-001`'s notes hold `- "First.\nSecond."`
- **AND** `qctl fmt` runs
- **THEN** the notes hold `- First.` then `- Second.`

#### Scenario: A note of only blank lines stays one item

- **WHEN** `DEMO-001`'s notes hold a `- |+` block whose lines are all empty
- **AND** `qctl fmt` runs
- **THEN** the notes still hold one item, written on one line
- **AND** `qctl show DEMO-001 --format json` reports the same note text as before

### Requirement: A verb rewriting notes splits a multi-line note

Under `list_items: line`, a verb that rewrites a row's whole block `notes` list, such as `qctl edit -x note:N` or `qctl edit -R`, SHALL write each note whose text holds line breaks as one item per line, as `fmt` does, and keep a note with no other line as one item.

#### Scenario: Removing a note splits a literal neighbour

- **WHEN** `DEMO-001`'s notes hold `- old` followed by a `- |-` block with the lines `one` and `two`
- **AND** `qctl edit DEMO-001 -x note:1` runs
- **THEN** the notes hold `- one` then `- two`

### Requirement: The instructions say how to date a note

`qctl instructions` SHALL tell the operator to write a note's date followed by a comma, never a colon, because a colon followed by a space, or a space followed by `#`, inside a line forces YAML quoting, and never to rewrite existing note text to achieve it.

#### Scenario: The instructions carry the date rule

- **WHEN** `qctl instructions` runs
- **THEN** its output says to follow a note's date with a comma, not a colon
- **AND** it says not to rewrite existing note text for this
