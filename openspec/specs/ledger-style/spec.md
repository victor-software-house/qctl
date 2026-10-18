# ledger-style Specification

## Purpose

How `fmt` applies the style a ledger declares, beyond the whitespace nobody chose.

## Requirements

### Requirement: A ledger declares how its rows are separated

The `style` block SHALL accept `row_spacing`, with the values `as_written` and
`blank_line`. A ledger that omits it SHALL behave as `as_written`, and `fmt`
SHALL leave the space between its rows as today's whitespace rule leaves it.

#### Scenario: A ledger without the option keeps its spacing

- **WHEN** a `tasks.yaml` with no `row_spacing` holds two queue rows with no
  blank line between them
- **AND** `qctl fmt` runs
- **THEN** the two rows stay adjacent

#### Scenario: An unknown value is refused

- **WHEN** a `tasks.yaml` declares `style: {row_spacing: double}`
- **THEN** `qctl check` fails and names `row_spacing`

### Requirement: fmt keeps one blank line between rows when the style asks

With `row_spacing: blank_line`, `qctl fmt` SHALL leave exactly one blank line
between each pair of consecutive rows in the queue, the archive, and the
horizon. It SHALL NOT put a blank line between a list's key and its first row,
and SHALL leave the space after a list's last row as written. The blank line
SHALL go above the comments a row owns. `qctl fmt --check` SHALL name the
missing separator and write nothing.

#### Scenario: A missing separator is inserted

- **WHEN** a `tasks.yaml` declares `style: {row_spacing: blank_line}` and its
  queue holds `DEMO-001` followed directly by `DEMO-002`
- **AND** `qctl fmt` runs
- **THEN** one blank line separates `DEMO-001` from `DEMO-002`
- **AND** no blank line follows `queue:`

#### Scenario: A doubled separator becomes one

- **WHEN** a `tasks.yaml` declares `style: {row_spacing: blank_line}` and two
  archive rows are separated by two blank lines
- **AND** `qctl fmt` runs
- **THEN** one blank line separates them

#### Scenario: The blank line goes above a row's comment

- **WHEN** a `tasks.yaml` declares `style: {row_spacing: blank_line}` and the
  line directly above `- id: DEMO-002` is `# waiting on review`, directly
  after `DEMO-001`
- **AND** `qctl fmt` runs
- **THEN** the blank line sits between `DEMO-001` and the comment, and the
  comment stays directly above `DEMO-002`

#### Scenario: check mode names the gap

- **WHEN** a `tasks.yaml` declares `style: {row_spacing: blank_line}` and two
  queue rows are adjacent
- **AND** `qctl fmt --check` runs
- **THEN** it exits non-zero and the file is unchanged

### Requirement: A kept block scalar is never extended

`fmt` SHALL NOT insert a blank line after a row whose text ends in a block
scalar that keeps its trailing lines (`|+` or `>+`), because that line would
become part of the value.
The rule applies to whatever block scalar ends the row: a row scalar such as
`outcome`, or, under `list_items: auto`, a list item.

#### Scenario: A row ending in a kept block scalar

- **WHEN** a `tasks.yaml` declares `style: {row_spacing: blank_line}` and the
  last key of `DEMO-001` is `outcome: |+`, its block directly followed by
  `- id: DEMO-002`
- **AND** `qctl fmt` runs
- **THEN** the file is unchanged and `DEMO-001`'s outcome reads back the same

### Requirement: A ledger declares its list-item layout

The `style` block SHALL accept `list_items`, with the values `line` and `auto`, which name yamled's one-line and automatic layouts.
A ledger that omits it SHALL behave as `line`.
Every writer and `fmt` SHALL lay out list items as it says.

#### Scenario: A ledger without the option gets the line layout

- **WHEN** a `tasks.yaml` with no `list_items` holds a row whose notes include a `- >2-` block
- **AND** `qctl fmt` runs
- **THEN** that note is written on one line

#### Scenario: An unknown layout is refused

- **WHEN** a `tasks.yaml` declares `style: {list_items: folded}`
- **THEN** `qctl check` fails and names `list_items`

### Requirement: A ledger declares its quotes

The `style` block SHALL accept `quotes`, with the values `auto`, `single`, and `double`, which name yamled's quote preferences.
A ledger that omits it SHALL behave as `auto`.
Every string a verb writes SHALL be plain when it reads back unquoted, and quoted as `quotes` says otherwise.

#### Scenario: The automatic quotes double-quote an apostrophe

- **WHEN** a `tasks.yaml` with no `quotes` gets `qctl edit DEMO-001 -t "It's done: mostly"`
- **THEN** the title line reads `title: "It's done: mostly"`

#### Scenario: Single quotes when the ledger asks

- **WHEN** a `tasks.yaml` declares `style: {quotes: single}`
- **AND** `qctl edit DEMO-001 -n "2026-10-15: it's blocked"` runs
- **THEN** the appended line reads `- '2026-10-15: it''s blocked'`

#### Scenario: An unknown quote preference is refused

- **WHEN** a `tasks.yaml` declares `style: {quotes: backtick}`
- **THEN** `qctl check` fails and names `quotes`

### Requirement: fmt re-quotes strings to the declared quotes

`qctl fmt` SHALL rewrite each string scalar written on one line whose quoting differs from what the declared `quotes` write, keeping its value.
It SHALL leave block scalars, keys, values that are not strings, and the `style` block as written.
`qctl fmt --check` SHALL fail on such a scalar and write nothing.

#### Scenario: A doubled quote becomes double quotes

- **WHEN** a row's note reads `- 'it''s here: now'` and the ledger declares no `quotes`
- **AND** `qctl fmt` runs
- **THEN** the note reads `- "it's here: now"`
- **AND** `qctl show DEMO-001 --format json` reports the same note text as before

#### Scenario: A needlessly quoted stamp becomes plain

- **WHEN** an archived row reads `completed: "2026-08-19T10:00:00"`
- **AND** `qctl fmt` runs
- **THEN** the line reads `completed: 2026-08-19T10:00:00`

#### Scenario: A string that agrees is untouched

- **WHEN** every one-line string is written as the declared quotes write it
- **AND** `qctl fmt --check` runs
- **THEN** it exits zero and the file is unchanged
