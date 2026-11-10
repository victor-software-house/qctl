# Tasks

The build host for every check below is the pull request's CI.

## 1. Schema and upgrade

- [x] 1.1 Rename `HorizonTask::kind` to `reason` and the `Kind` enum to `Reason`, move `VERSION` to 6, and regenerate `schema/tasks.schema.json` with `mise run schema`. Verify `schema_file` passes in CI and the generated schema names `reason` and `Reason`.
  Evidence, 2026-11-10: shipped in [#174][qctl#174]; `schema_file` passes, and the schema names `reason` and `Reason`.
- [x] 1.2 Teach `fmt` to rename each horizon `kind` key in place when it upgrades schema 3, 4, or 5, and make every other verb refuse those versions with a message that names `qctl fmt`. Verify `fmt_renames_the_horizon_kind_of_schema_5_in_place`, `other_verbs_refuse_schema_5_and_name_fmt`, and `fmt_upgrades_schema_4_by_one_line` pass in CI.
  Evidence, 2026-11-10: shipped in [#174][qctl#174]; the three named tests pass.

## 2. Flags and output

- [x] 2.1 Replace `-k/--kind` with `-r/--reason` on `add`, `park`, and `edit`, and say `reason` in the `status` horizon table and every message. Verify `park_reads_its_reason_from_qctl_reason_and_refuses_kind`, the `rows_horizon` cases, and the usage snapshot pass in CI.
  Evidence, 2026-11-10: shipped in [#174][qctl#174]; `park_reads_its_reason_from_qctl_reason_and_refuses_kind`, the `rows_horizon` cases and the usage snapshot pass.
- [x] 2.2 Move every test ledger to version 6 and `reason:`, and accept the changed mutation snapshots from a test run with `mise run snapshots`. Verify the mutation snapshots pass in CI.
  Evidence, 2026-11-10: [#174][qctl#174] moved every test ledger and mutation snapshot to version 6.

## 3. Contracts

- [x] 3.1 Update `src/instructions.md`, `skills/qctl/SKILL.md`, the `.ctl/operator/` templates, `AGENTS.md`, and `docs/adoption-report.md`. Verify `operator_docs` passes in CI.
  Evidence, 2026-11-10: [#174][qctl#174] changed all five files; `operator_docs` passes.
- [x] 3.2 Verify `openspec validate --all --strict` passes through the `specs` task in CI.
  Evidence, 2026-11-10: the `specs` task ran it in the CI of [#174][qctl#174] and of main at 802ac0d ([run 42856092183][qctl-main-ci]).

## 4. Rollout

- [x] 4.1 Upgrade this repository's `tasks.yaml` to version 6 with the built binary's `qctl fmt`. Verify the diff renames the 2 horizon `kind:` keys and the version line, and changes no other value.
  Evidence, 2026-11-10: [#174][qctl#174] moved `tasks.yaml` to version 6 and renamed both horizon keys; its one other line re-quotes an outcome without changing its value, as `qctl fmt` does by design ([`src/format.rs`][format-rs]).
- [ ] 4.2 File the queue row that names this change once the open ledger pull requests merge. Verify `qctl check` passes on this repository.
- [ ] 4.3 After the release, archive this change and archive its row with `qctl close-from-openspec`. Verify `qctl check` passes on this repository.

[qctl#174]: https://github.com/victor-software-house/qctl/pull/174
[format-rs]: ../../../src/format.rs
[qctl-main-ci]: https://github.com/victor-software-house/qctl/actions/runs/42856092183
