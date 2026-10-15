# Spec Delta

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
