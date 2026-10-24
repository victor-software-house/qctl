# Tasks

- [x] 1.1 Move to ctl-core 0.6.16; proof: the full suite and `operator_docs` pass on the build host
  - 2026-10-24 UTC: [PR 106][qctl#106] moved to ctl-core 0.6.16 and merged
    as [85ca4ed] on 2026-10-13. Its CI Verify, which runs the full suite
    with `operator_docs`, passed on its last push, [ed21aaf]. Released as
    qctl 0.5.2 by [PR 107][qctl#107].
- [x] 1.2 Forced-colour JSON test over success, a domain error, a parse error, and an unknown verb; proof: `json_output_holds_no_escape_byte_under_forced_colour` on the build host
  - 2026-10-24 UTC: the test at [85ca4ed] runs `status` (success), `check`
    on a broken ledger (domain error), `status --bogus` (parse error), and
    `nosuchverb` (unknown verb) under `CLICOLOR_FORCE=1`, `FORCE_COLOR=3`,
    and `COLORTERM=truecolor`, and passed in the same CI Verify run.

[qctl#106]: https://github.com/victor-software-house/qctl/pull/106
[qctl#107]: https://github.com/victor-software-house/qctl/pull/107
[85ca4ed]: https://github.com/victor-software-house/qctl/commit/85ca4ed
[ed21aaf]: https://github.com/victor-software-house/qctl/commit/ed21aaf
