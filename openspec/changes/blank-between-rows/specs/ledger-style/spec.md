## ADDED Requirements

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

#### Scenario: A row ending in a kept block scalar

- **WHEN** a `tasks.yaml` declares `style: {row_spacing: blank_line}` and the
  last note of `DEMO-001` is a `|+` block directly followed by `- id: DEMO-002`
- **AND** `qctl fmt` runs
- **THEN** the file is unchanged and `DEMO-001`'s note reads back the same
