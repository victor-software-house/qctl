# Read ledgers through ctl-core and edit them through yamled

## Why

qctl carries four YAML crates (`serde_yml` 0.0.12, `yaml_serde` 0.10,
`yamlpath` 1.29, and `yamlpatch` 1.29, which pulls in tree-sitter) and about
970 lines of its own YAML text handling under `src/document/`. A schema error
from `qctl check` names a JSON pointer but no line, and a value error from a
verb names the field but no line. ctl-core 0.6.6 now ships the shared declared
input layer, and yamled 0.0.2 ships the format-preserving edits a ledger needs.
This change is queue row QCTL-040.

## What Changes

1. Every ledger read (`status`, `show`, `check`, `fmt`, and each verb's read
   back) parses through ctl-core's `input` feature: strict booleans, duplicate
   and merge keys refused, and every problem reported with its file and line.
2. `qctl check` places each JSON Schema error on its line through yamled's
   location index: `tasks.yaml:12:5: /queue/0/id: ...`.
3. Every ledger edit goes through yamled: row moves, key edits, list rewrites,
   section order, and re-indent. The comment-ownership, list-rewrite, and
   empty-queue workarounds in `src/document.rs` go, because yamled owns those
   rules.
4. `serde_yml`, `yaml_serde`, `yamlpath`, and `yamlpatch` leave the
   dependency tree.
5. **Output change:** a value error from a verb and a schema error from
   `check` now start with `file:line:column:`.

## Capabilities

### New Capabilities

- `ledger-yaml`: how qctl reads a ledger and where it reports a problem, and how
  an edit keeps every byte it does not name.

### Modified Capabilities

None.

## Impact

1. Code: `src/ledger.rs`, `src/check.rs`, `src/format.rs`, `src/mutate.rs`,
   `src/document.rs` and `src/document/`.
2. Dependencies: ctl-core `=0.6.6` with `input`; yamled `0.0.2`; four YAML
   crates removed.
3. Tests: the 33 mutation fixtures keep their accepted snapshots, or a changed
   snapshot names the byte it now keeps.
4. Delivery: two stacked pull requests. The first moves every read; the second
   moves every edit.
