# Proposal

## Why

qctl pins ctl-core 0.7.0, and ctl-core 0.8.0-rc.2 is published ([changelog][changelog]).
Row QCTL-057 asks for `qctl add` and `qctl edit` to take a row from a JSON or YAML document, and ctl-core 0.8 adds that document to every command behind `input-document`.
The 0.8 releases also change output a script reads: piped output defaults to `tsv`, and a JSON failure goes to stderr.
The queue row is QCTL-060.

An agent that adds a row today writes a long flag list, with `-a` and `-n` repeated per item.

## What Changes

1. Pin ctl-core `=0.8.0-rc.2`, with `input-document` and `input-json`, and migrate every 0.8 break by the changelog's migration notes.
2. Every command takes `--input <FILE>` and `--input-format <NAME>`, also read from `QCTL_INPUT` and `QCTL_INPUT_FORMAT`.
3. Output that ctl-core 0.8 changed changes in qctl the same way.
4. The skill and the installed instructions describe the input document and the piped default.

## Capabilities

### New Capabilities

- `input-document`: qctl commands read their arguments from a document.

### Modified Capabilities

- `presentation`: the piped default format, and the fallback width for piped `pretty` output.

## Impact

1. Code: `Cargo.toml`, `Cargo.lock`, `tests/cli.rs`, the tests and their snapshots.
2. Docs: `skills/qctl/SKILL.md`, `src/instructions.md` and their templates.
3. Release: a `qctl: minor` changeset, shipped as `0.8.0-rc.1` with `prerelease: rc`.
4. A batch of rows in one document, the second acceptance of QCTL-057, stays with that row.

[changelog]: https://github.com/victor-software-house/ctl-core/blob/4453827c9771269b450b70a18ac8fe2a4cffd4c6/CHANGELOG.md
