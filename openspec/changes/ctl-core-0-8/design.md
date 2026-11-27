# Design

## Context

ctl-core 0.8 adds the input document behind `input-document`: `App` adds `--input` and `--input-format` to every command, and a document's keys are long option names and positional ids ([changelog][changelog]).
qctl's `add` takes `-a` and `-n` once per item, so a row with five notes needs ten arguments.

## Decisions

### 1. Take ctl-core's piped default

Piped output becomes `tsv`, as ctl-core 0.8 makes it for every CLI.
A script that wants the old table passes `--format pretty` or sets `QCTL_FORMAT=pretty`.

Alternative considered: keep `pretty` piped through `Formats::builtin().piped_default(FormatName::PRETTY)`.
It lost because ctl-core chose `tsv` so that a script reads the columns a person sees.

### 2. One row per document

A document fills one invocation, so `add --input` adds one row and `edit --input` edits one.
A list value, such as `acceptance` or `note`, is a YAML or JSON list.

Alternative considered: a qctl format for a batch of rows.
It lost for this change because it needs qctl's own transaction over many rows, which QCTL-057 keeps.

### 3. Conflicts drop the losing key

qctl keeps ctl-core's default, where the command line, then the environment, then the document wins, and a losing key is dropped.
`App::input_conflicts(InputConflicts::Fail)` is the override qctl does not take.

### 4. A minor candidate

The changeset is `qctl: minor`, and `.ctl/ver.yaml` sets `prerelease: rc` on the `qctl` package, so the Version PR ships `0.8.0-rc.1`.
qctl took ctl-core 0.7.0 as the minor 0.7.0 the same way.

## Risks

1. A caller that parsed piped pretty output breaks.
   This repository's workflows and tasks read no piped qctl table.

[changelog]: https://github.com/victor-software-house/ctl-core/blob/4453827c9771269b450b70a18ac8fe2a4cffd4c6/CHANGELOG.md
