# Spec Delta

## ADDED Requirements

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
It SHALL leave block scalars, keys, and values that are not strings as written.
`qctl fmt --check` SHALL fail on such a scalar and write nothing.

#### Scenario: A doubled quote becomes double quotes

- **WHEN** a row's note reads `- 'it''s here: now'` and the ledger declares no `quotes`
- **AND** `qctl fmt` runs
- **THEN** the note reads `- "it's here: now"`
- **AND** `qctl show DEMO-001 --format json` reports the same note text as before

#### Scenario: A needlessly quoted stamp becomes plain

- **WHEN** an archived row reads `completed: "2026-08-19T10:00:00+00:00"`
- **AND** `qctl fmt` runs
- **THEN** the line reads `completed: 2026-08-19T10:00:00+00:00`

#### Scenario: A string that agrees is untouched

- **WHEN** every one-line string is written as the declared quotes write it
- **AND** `qctl fmt --check` runs
- **THEN** it exits zero and the file is unchanged

## MODIFIED Requirements

### Requirement: A kept block scalar is never extended

`fmt` SHALL NOT insert a blank line after a row whose text ends in a block
scalar that keeps its trailing lines (`|+` or `>+`), because that line would
become part of the value.
A list item is never such a block after `fmt`, which writes every list item on
one line, so the rule applies to a row scalar such as `outcome`.

#### Scenario: A row ending in a kept block scalar

- **WHEN** a `tasks.yaml` declares `style: {row_spacing: blank_line}` and the
  last key of `DEMO-001` is `outcome: |+`, its block directly followed by
  `- id: DEMO-002`
- **AND** `qctl fmt` runs
- **THEN** the file is unchanged and `DEMO-001`'s outcome reads back the same
