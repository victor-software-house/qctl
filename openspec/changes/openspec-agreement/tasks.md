# Tasks

The build host for every check below is the pull request's CI.

## 1. Layout reader

- [ ] 1.1 Add `src/openspec.rs`: open change names, archive folders by change name, and the change a `plan:` names. Verify the "A row names a change through its plan path" scenarios pass as unit tests in CI.

## 2. check

- [ ] 2.1 Add the three rules (finished row with an open change, plan pointing into an archived change, change no row names) to `check`, applied only when an `openspec/` directory exists. Verify every scenario of the three `check` requirements and "The agreement rules run exactly where OpenSpec does" passes over a scratch ledger in CI.
- [ ] 2.2 Verify `qctl check --format json` carries the new problems with `CLICOLOR_FORCE=1` set and no ESC byte, in the existing plain-JSON test.

## 3. archive and keep_change_open

- [ ] 3.1 Add the optional `keep_change_open` field to `ArchivedTask`, move `VERSION` to 5, teach `fmt` the 4 to 5 upgrade and the other verbs the refusal, and regenerate `schema/tasks.schema.json`. Verify the scenarios of "A ledger that may carry keep_change_open declares schema version 5" pass, and that every test ledger declares version 5. This repository's own `tasks.yaml` moves to version 5 in task 6.3, because its `q check` runs the released qctl until the Version PR moves that pin.
- [ ] 3.2 Make `archive` and `close-from-git` refuse the last row of an open change, unless `--keep-change-open` gives a reason. Verify the three scenarios of "archive refuses the last row of an open change without a reason to keep it open" pass, with mutation snapshots for the refused case and the keep_change_open case.

## 4. close-from-openspec

- [ ] 4.1 Add the verb, its report, and its presentation. Verify the scenarios of "close-from-openspec closes and repoints rows of archived changes" and "close-from-openspec leaves what it cannot decide" pass, with a mutation snapshot for the archive-and-repoint case.

## 5. Contracts

- [ ] 5.1 Update `src/instructions.md`, `skills/qctl/SKILL.md`, the `.ctl/operator/` templates, and `AGENTS.md` for the link rule, the new rules, `--keep-change-open`, schema version 5, and `close-from-openspec`. Verify `operator_docs` passes in CI.
- [ ] 5.2 Verify `openspec validate openspec-agreement --strict` passes in CI.

## 6. Rollout

- [x] 6.1 Release qctl as a minor (0.6.0) through the Version PR. Verify the release tag exists and its binaries are attached.
  - 2026-10-24 UTC: [PR 153][qctl#153] released [v0.6.0] with
    `qctl_0.6.0_linux_x64.tar.gz`, `qctl_0.6.0_macos_arm64.tar.gz`, and
    `tasks.schema.json` attached.
- [ ] 6.2 Run the released `qctl check` in each ctl repository beside an `openspec/` tree and fix what it reports in that repository's qctl bump pull request. Verify each repository's CI runs `qctl check` green on the bump.
- [ ] 6.3 After the release, run `qctl fmt` on this repository's `tasks.yaml` to reach version 5, archive this change, and point QCTL-051 at the archived proposal with `qctl close-from-openspec`. Verify `qctl check` passes on this repository.

[qctl#153]: https://github.com/victor-software-house/qctl/pull/153
[v0.6.0]: https://github.com/victor-software-house/qctl/releases/tag/v0.6.0
