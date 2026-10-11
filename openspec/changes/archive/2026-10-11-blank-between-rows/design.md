# Design: blank lines between rows

## Decisions

### 1. An enum, not a boolean

`row_spacing: as_written | blank_line`. A boolean named `blank_between_rows`
was considered. It lost because the next request, rows that must be tight, is
a third value of the same question, and an enum takes it without a second key
that could contradict the first.

### 2. Rows are found through yamled's owned ranges

`yamled::Node::owned` gives each row's lines: the comments above it, its
entry, and its value. The gap between one row's owned range and the next is
what `fmt` rewrites, so a blank line lands above a row's comments without qctl
deciding which comment belongs to which row. Scanning for `- id:` lines was
considered. It lost because it cannot tell a row's comment from a comment that
belongs to no row, which yamled already decides for every other edit.

### 3. Only whitespace gaps are rewritten

A gap holding anything other than blank lines is left as written. Today that
cannot happen between two rows, because yamled gives every comment above a row
to that row; the rule keeps `fmt` from deleting text if that ever changes.

### 4. Insertion only; collapsing stays where it is

`tidied` already collapses a run of blank lines to one. The new pass only adds
a blank line where a gap has none, and runs before `tidied`, so the two rules
never disagree.

### 5. A kept block scalar is skipped

A blank line after a `|+` or `>+` block is part of its value. The pass reads
the previous row's lines with the existing `block_scalar` detector and skips
the gap when the row ends inside such a block. Moving the blank line inside
the next row's indentation was considered. It lost because YAML would still
fold it into the block.

## Override and disable

The option is off by default (`as_written`). A ledger turns it on with
`style: {row_spacing: blank_line}` and off by removing it. `normalize_on_write`
applies it on every verb, as it applies every other style rule.
