# Tasks

Compile, format, lint, and test tasks run on the build host with `mise run verify`.

## 1. Adopt

- [x] 1.1 Pin ctl-core `=0.6.3` in both dependency tables; proof: `Cargo.lock` resolves ctl-core 0.6.3
- [x] 1.2 Move row ids to `Text::id` and `Table::id_column`, keeping paths as tokens; proof: `rg 'token\(' src/presentation.rs` lists only paths
- [x] 1.3 Run text-matching integration tests at `COLUMNS=1000`; proof: `mise run verify` passes
- [x] 1.4 Add `piped_show_without_columns_stays_within_80_columns`; proof: the test passes and failed on ctl-core 0.6.1, which did not wrap headings
- [x] 1.5 Close QCTL-037 when the pull request merges; proof: `qctl check` passes
