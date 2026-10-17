# Tasks

- [x] 1.1 Bind `QCTL_<LONG>` on the root parser and drop `TASKS_LEDGER`; proof: `variables_set_flags_under_the_command_line` and `the_ledger_comes_from_qctl_file` pass in the pull request's CI
- [x] 1.2 Clear every `QCTL_*` variable in the test helper; proof: the suite passes in CI
- [x] 1.3 Update the Usage snapshot, operator docs, README, and AGENTS.md; proof: `usage_spec_is_the_mounted_q_grammar` and `operator_docs` pass in CI
- [x] 1.4 Patch changeset naming the `TASKS_LEDGER` rename; proof: the pull request's CI is green

2026-10-17 21:05 UTC: [run 39068830152](https://github.com/victor-software-house/qctl/actions/runs/39068830152) on [qctl#129](https://github.com/victor-software-house/qctl/pull/129) passed Verify, including `variables_set_flags_under_the_command_line`, `the_ledger_comes_from_qctl_file`, `usage_spec_is_the_mounted_q_grammar`, and `operator_docs`.
