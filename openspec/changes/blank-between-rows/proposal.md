# Proposal: blank lines between rows

Queue row: QCTL-014.

## Why

Every ctl ledger sets its rows apart with one blank line: 119 rows across five
ledgers, none missing one today. Nothing keeps it that way. `fmt` collapses a
doubled blank line, but it never inserts a missing one, so a lost separator is
found by eye. QCTL-013 repaired two by hand after a cut row took them along.

## What Changes

1. A new style option, `row_spacing`, says how rows in a list are separated:
   `as_written` (the default, today's behaviour) or `blank_line`.
2. With `blank_line`, `fmt` puts exactly one blank line between consecutive
   rows of each list. A missing separator is inserted; a doubled one is already
   collapsed by today's whitespace rule.
3. The first row of a list never gets a blank line above it, and the space
   after a list's last row is left as written.
4. A blank line goes above the comments a row owns, not between them and the
   row.
5. A row whose last value is a block scalar that keeps its trailing lines
   (`|+` or `>+`) is left as written: a blank line after it would become part
   of that value.

## Capabilities

### New Capabilities

- `ledger-style`: how `fmt` applies a ledger's declared style, starting with
  row spacing.

### Modified Capabilities

None.

## Impact

1. Code: `src/schema.rs` (the option), `src/format.rs` (applies it),
   `src/document.rs` (rows found through yamled's owned ranges).
2. The generated `schema/tasks.schema.json` gains the option. A ledger without
   it is unchanged, so this is a minor change, not a migration.
3. Docs: the style table in `src/instructions.md`, and the fmt rule in the
   operator skill that says it never inserts a blank line.
4. Tests: new `fmt` mutation scenarios.
