# Tasks

## 1. Reads through ctl-core input

- [x] 1.1 `load`, `read`, `load_value`, `fmt`, and each verb's read back parse
  through `ctl_core::input::Input`. Proof: `mise run verify` on the build host.
- [x] 1.2 `qctl check` prefixes each schema error with `file:line:column:`.
  Proof: `a_schema_error_from_check_names_its_line` in `tests/cli.rs`.
- [x] 1.3 `serde_yml` is gone. Proof: `cargo tree -e normal` on the build host
  names no `serde_yml`.

2026-09-28 (-03:00): `mise run verify` passed on the build host with the
reads moved, and `cargo tree -e normal` there names no `serde_yml`. yamled
resolves twice (0.0.1 through ctl-core 0.6.6, 0.0.2 directly) until ctl-core
moves to yamled 0.0.2.

## 2. Edits through yamled

- [x] 2.1 `src/document.rs` edits through yamled, and the comment-ownership,
  list-rewrite, and empty-queue workarounds are gone. Proof: the 33 mutation
  snapshots pass on the build host, or each changed snapshot names the byte it
  now keeps.
- [x] 2.2 `yaml_serde`, `yamlpath`, and `yamlpatch` are gone. Proof:
  `cargo tree -e normal` names none of them.
- [x] 2.3 AGENTS.md, `src/instructions.md`, and `skills/qctl/SKILL.md` say
  what is now true. Proof: review of the diff.

2026-09-29 (-03:00): `mise run verify` passed on the build host with every
edit on yamled 0.0.3 and ctl-core 0.6.7. 32 of the 33 mutation snapshots are
unchanged. `edit-moves-a-row-before` changed: the list is written with no blank
line between rows, and the moved row now keeps that spacing where the old
splice added a blank line. `cargo tree -e normal` names none of `serde_yml`,
`yaml_serde`, `yamlpath`, `yamlpatch`, or a `tree-sitter` crate, and resolves
one yamled. The port found a yamled defect, a note written as `|+` losing a
line break when the next note was pushed, fixed in yamled 0.0.3
([yamled#8]).

The review of the whole stack then found that a note ending in two line
breaks, written as `|+`, left its row unmovable: yamled refused to take or
reorder it, and `fmt` failed because `reorder_sections` had lost its early
return. yamled 0.0.4 moves such an item with its kept lines ([yamled#10]),
`reorder_sections` returns early again, and
`a_row_whose_last_note_keeps_its_line_breaks_still_moves` in `tests/notes.rs`
starts, formats, and archives such a row. It fails on yamled 0.0.3.

[yamled#8]: https://github.com/victor-software-house/yamled/pull/8
[yamled#10]: https://github.com/victor-software-house/yamled/pull/10
