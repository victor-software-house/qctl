# Tasks

The build host for every check below is the pull request's CI.

## 1. Layout reader

- [x] 1.1 Add `src/openspec.rs`: open change names, archive folders by change name, and the change a `plan:` names. Verify the "A row names a change through its plan path" scenarios pass as unit tests in CI.
  - 2026-10-24 UTC: [PR 151][qctl#151] adds `src/openspec.rs` with unit
    tests `an_open_change_is_named_by_its_folder`,
    `an_archived_change_drops_its_date`, `other_plans_name_no_change`, and
    `an_archive_folder_needs_a_date_and_a_name`; they passed in [CI on
    537cec6][run-151].

## 2. check

- [x] 2.1 Add the three rules (finished row with an open change, plan pointing into an archived change, change no row names) to `check`, applied only when an `openspec/` directory exists. Verify every scenario of the three `check` requirements and "The agreement rules run exactly where OpenSpec does" passes over a scratch ledger in CI.
  - 2026-10-24 UTC: `tests/openspec.rs` from [PR 151][qctl#151] covers each
    rule over a scratch ledger beside an `openspec/` tree, and
    `a_ledger_without_openspec_gets_no_agreement_rule` the no-tree case; all
    passed in [CI on 537cec6][run-151].
- [x] 2.2 Verify `qctl check --format json` carries the new problems with `CLICOLOR_FORCE=1` set and no ESC byte, in the existing plain-JSON test.
  - 2026-10-24 UTC: [PR 157][qctl#157] adds a ledger beside an `openspec/`
    tree to `json_output_holds_no_escape_byte_under_forced_colour`, which
    asserts the JSON names the unnamed change and holds no ESC byte; it
    passed in [CI on c10bbe7][run-157].

## 3. archive and keep_change_open

- [x] 3.1 Add the optional `keep_change_open` field to `ArchivedTask`, move `VERSION` to 5, teach `fmt` the 4 to 5 upgrade and the other verbs the refusal, and regenerate `schema/tasks.schema.json`. Verify the scenarios of "A ledger that may carry keep_change_open declares schema version 5" pass, and that every test ledger declares version 5. This repository's own `tasks.yaml` moves to version 5 in task 6.3, because its `q check` runs the released qctl until the Version PR moves that pin.
  - 2026-10-24 UTC: [PR 151][qctl#151] moves `VERSION` to 5 and every test
    ledger to version 5, and `fmt_upgrades_schema_4_by_one_line` passed in
    [CI on 537cec6][run-151]. This repository's ledger moved to version 5 in
    [PR 155][qctl#155].
- [x] 3.2 Make `archive` and `close-from-git` refuse the last row of an open change, unless `--keep-change-open` gives a reason. Verify the three scenarios of "archive refuses the last row of an open change without a reason to keep it open" pass, with mutation snapshots for the refused case and the keep_change_open case.
  - 2026-10-24 UTC: `archive_refuses_the_last_row_of_an_open_change` asserts
    the refused ledger stays byte-identical,
    `a_trailer_cannot_close_the_last_row_of_an_open_change` covers
    `close-from-git`, and both passed in [CI on 537cec6][run-151]. The
    mutation snapshot `archive-keeps-a-change-open-with-a-reason` passed in
    [CI on c10bbe7][run-157]. The mutation harness records only verbs that
    succeed, so the refused case is the byte-identical assertion instead of
    a snapshot.

## 4. close-from-openspec

- [x] 4.1 Add the verb, its report, and its presentation. Verify the scenarios of "close-from-openspec closes and repoints rows of archived changes" and "close-from-openspec leaves what it cannot decide" pass, with a mutation snapshot for the archive-and-repoint case.
  - 2026-10-24 UTC: the `close_from_openspec_*` tests passed in [CI on
    537cec6][run-151], and the mutation snapshot
    `close-from-openspec-closes-and-repoints` passed in [CI on
    c10bbe7][run-157].

## 5. Contracts

- [x] 5.1 Update `src/instructions.md`, `skills/qctl/SKILL.md`, the `.ctl/operator/` templates, and `AGENTS.md` for the link rule, the new rules, `--keep-change-open`, schema version 5, and `close-from-openspec`. Verify `operator_docs` passes in CI.
  - 2026-10-24 UTC: [PR 151][qctl#151] updates the instructions, skill,
    operator templates, and `AGENTS.md`; `operator_docs` passed in [CI on
    537cec6][run-151].
- [x] 5.2 Verify `openspec validate openspec-agreement --strict` passes in CI.
  - 2026-10-24 UTC: the `specs` task runs `openspec validate --all
    --strict`, which passed in [CI on 537cec6][run-151].

## 6. Rollout

- [x] 6.1 Release qctl as a minor (0.6.0) through the Version PR. Verify the release tag exists and its binaries are attached.
  - 2026-10-24 UTC: [PR 153][qctl#153] released [v0.6.0] with
    `qctl_0.6.0_linux_x64.tar.gz`, `qctl_0.6.0_macos_arm64.tar.gz`, and
    `tasks.schema.json` attached.
- [x] 6.2 Run the released `qctl check` in each ctl repository beside an `openspec/` tree and fix what it reports in that repository's qctl bump pull request. Verify each repository's CI runs `qctl check` green on the bump.
  - 2026-10-24 UTC: the released `qctl check` ran on each ctl repository's
    main, and each fix landed in that repository's qctl 0.6.0 pull request
    with `qctl check` green in its CI: [verctl#211], [ctl-core#166],
    [yamled#57], [forkctl#135], [unixd#51], and [PR 155][qctl#155] here,
    plus the private template repository's ledger. bun-release has no
    ledger.
- [x] 6.3 After the release, run `qctl fmt` on this repository's `tasks.yaml` to reach version 5, archive this change, and point QCTL-051 at the archived proposal with `qctl close-from-openspec`. Verify `qctl check` passes on this repository.
  - 2026-10-24 UTC: [PR 155][qctl#155] ran `qctl fmt` to version 5. [PR
    157][qctl#157] archives this change, and `qctl close-from-openspec`
    archives QCTL-051 with the archive folder as evidence; `qctl check`
    passes.

[qctl#153]: https://github.com/victor-software-house/qctl/pull/153
[v0.6.0]: https://github.com/victor-software-house/qctl/releases/tag/v0.6.0
[qctl#151]: https://github.com/victor-software-house/qctl/pull/151
[qctl#155]: https://github.com/victor-software-house/qctl/pull/155
[qctl#157]: https://github.com/victor-software-house/qctl/pull/157
[run-151]: https://github.com/victor-software-house/qctl/actions/runs/40056778404
[run-157]: https://github.com/victor-software-house/qctl/actions/runs/40062533627
[verctl#211]: https://github.com/victor-software-house/verctl/pull/211
[ctl-core#166]: https://github.com/victor-software-house/ctl-core/pull/166
[yamled#57]: https://github.com/victor-software-house/yamled/pull/57
[forkctl#135]: https://github.com/victor-software-house/forkctl/pull/135
[unixd#51]: https://github.com/victor-software-house/unixd/pull/51
