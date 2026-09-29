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

- [ ] 2.1 `src/document.rs` edits through yamled, and the comment-ownership,
  list-rewrite, and empty-queue workarounds are gone. Proof: the 33 mutation
  snapshots pass on the build host, or each changed snapshot names the byte it
  now keeps.
- [ ] 2.2 `yaml_serde`, `yamlpath`, and `yamlpatch` are gone. Proof:
  `cargo tree -e normal` names none of them.
- [ ] 2.3 AGENTS.md, `src/instructions.md`, and `skills/qctl/SKILL.md` say
  what is now true. Proof: review of the diff.
