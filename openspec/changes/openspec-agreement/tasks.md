# Tasks

The build host for every check below is the pull request's CI.

## 1. Layout reader

- [ ] 1.1 Add `src/openspec.rs`: open change names, archive folders by change name, and the change a `plan:` names. Verify the "A row names a change through its plan path" scenarios pass as unit tests in CI.

## 2. check

- [ ] 2.1 Add the finished-row, archived-since, and orphan rules to `check`, applied only when an `openspec/` directory exists. Verify every scenario of the three `check` requirements and "The agreement rules run exactly where OpenSpec does" passes over a scratch ledger in CI.
- [ ] 2.2 Verify `qctl check --format json` carries the new problems with `CLICOLOR_FORCE=1` set and no ESC byte, in the existing plain-JSON test.

## 3. archive and the waiver

- [ ] 3.1 Add the optional `waiver` field to `ArchivedTask`, move `VERSION` to 5, teach `fmt` the 4 to 5 upgrade and the other verbs the refusal, and regenerate `schema/tasks.schema.json`. Verify the scenarios of "A ledger that may carry a waiver declares schema version 5" pass, and that this repository's `tasks.yaml` and `examples/` declare version 5.
- [ ] 3.2 Gate `archive` and `close-from-git` on the change, with `--waiver`. Verify the three scenarios of "archive refuses the last row of an open change without a waiver" pass, with mutation snapshots for the refused and waived cases.

## 4. close-from-openspec

- [ ] 4.1 Add the verb, its report, and its presentation. Verify the scenarios of "close-from-openspec closes and repoints rows of archived changes" and "close-from-openspec leaves what it cannot decide" pass, with a mutation snapshot for the archive-and-repoint case.

## 5. Contracts

- [ ] 5.1 Update `src/instructions.md`, `skills/qctl/SKILL.md`, the `.ctl/operator/` templates, and `AGENTS.md` for the link rule, the new rules, `--waiver`, schema version 5, and `close-from-openspec`. Verify `operator_docs` passes in CI.
- [ ] 5.2 Verify `openspec validate openspec-agreement --strict` passes in CI.

## 6. Rollout

- [ ] 6.1 Release qctl as a patch through the Version PR. Verify the release tag exists and its binaries are attached.
- [ ] 6.2 Run the released `qctl check` in each ctl repository beside an `openspec/` tree and fix what it reports in that repository's qctl bump pull request. Verify each repository's CI runs `qctl check` green on the bump.
- [ ] 6.3 Archive this change and point QCTL-051 at the archived proposal with `qctl close-from-openspec`. Verify `qctl check` passes on this repository.
