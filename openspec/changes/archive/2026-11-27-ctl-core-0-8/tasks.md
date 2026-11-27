# Tasks

## 1. The upgrade

- [x] 1.1 Pin ctl-core `=0.8.0-rc.2` in both dependency tables with `input-document` and `input-json` on the runtime entry, and take it past the cooldown with `mise run update:own`.
  Verify: `cargo tree -d` lists one copy of each ctl crate and of serde-saphyr.
  - 2026-11-27 (UTC): done in [776a955][qctl-pin], part of [PR 200][qctl#200], merged as [b4ff7f8][qctl-merge].
    [`Cargo.toml`][qctl-cargo] pins `=0.8.0-rc.2` in both tables, with `app`, `input`, `input-document`, `input-json` and `usage` on the runtime entry.
  - Departure: the lock moved with one `cargo update -p ctl-core@0.7.0 --precise 0.8.0-rc.2` under `CARGO_RESOLVER_INCOMPATIBLE_PUBLISH_AGE=allow`, not `mise run update:own`.
    `update:own` refuses the added crates erased-serde 0.4.11 and typeid 1.0.3, which no trusted owner publishes.
    The commit also raised yamled from 0.0.12 to 0.0.14, so the lock holds one serde-saphyr.
  - The lock diff of [776a955][qctl-pin] adds ctl-input, erased-serde and typeid, moves the four other ctl crates to 0.8.0-rc.2, and moves vsh-serde-saphyr from 0.0.3 to 0.0.6 and yamled from 0.0.12 to 0.0.14.
  - Evidence: [`Cargo.lock` at the merge commit][qctl-lock] has exactly one `[[package]]` entry for each of ctl-core, ctl-input, ctl-layout, ctl-term, ctl-text and vsh-serde-saphyr, which is what `cargo tree -d` would report.
    The PR description records `cargo tree -i vsh-serde-saphyr` showing one copy.
    `cargo tree -d` itself was not run while closing this change.
- [x] 1.2 Migrate every 0.8 break by the changelog's migration notes, and accept snapshots from a real run.
  Verify: `mise run verify` passes on the build host.
  - 2026-11-27 (UTC): the migration is [e9e6885][qctl-tests], which reads output as ctl-core 0.8 writes it and updates the usage snapshot.
    The PR description records `mise run verify` passing on the build host with 52 mutation scenarios.
    The [CI run][qctl-ci] on the PR head [61f341c][qctl-head] passed every job, including `verify / Build and test` and `verify / Verify`.
  - The repository holds 52 mutation scenarios and 52 mutation snapshots at the merge commit.
    The accept-then-check sequence (`INSTA_UPDATE=always`, then `INSTA_UPDATE=no`) ran on the build host and left no trace in the repository, so it is recorded here and not re-proven.

## 2. The input document

- [x] 2.1 Add the golden tests for `add --input` from JSON and `edit --input` from YAML.
  Verify: both scenarios pass as tests on the build host.
  - 2026-11-27 (UTC): [0b415f5][qctl-golden] adds the scenarios `add-from-a-json-document` (`add --input row.json`) and `edit-a-title-from-a-yaml-document` (`edit DEMO-001 --input edit.yaml`), each with its fixture and snapshot.
    Both ran inside the 52 scenarios of the passing [CI run][qctl-ci] and of the build-host `mise run verify` the PR description records.

## 3. Docs and release

- [x] 3.1 Describe the input document, its variables, the precedence and the piped default in the skill and the instructions, through their templates.
  Verify: the rendered files match their templates in `mise run verify`.
  - 2026-11-27 (UTC): [1bd2590][qctl-docs] edits both templates and both rendered files.
    [`skills/qctl/SKILL.md`][qctl-skill] and [`src/instructions.md`][qctl-instructions] name `--input`, `--input-format`, `QCTL_INPUT`, `QCTL_INPUT_FORMAT` and the `tsv` piped default.
    The byte comparison of each render with its template is the `operator_docs` test, which ran in the passing [CI run][qctl-ci].
- [x] 3.2 Set `prerelease: rc` on the `qctl` package and write a `qctl: minor` changeset naming the new flags and each output change.
  Verify: `openspec validate ctl-core-0-8 --strict` passes.
  - 2026-11-27 (UTC): [aa012fe][qctl-release-commit] sets `prerelease: rc` in [`.ctl/ver.yaml`][qctl-ver] and adds [the changeset][qctl-changeset] with `"qctl": minor`, which names `--input`, `--input-format`, `--json`, `--stream` and each output change.
    `openspec validate ctl-core-0-8 --strict` exited 0 before the archive.
  - The change shipped as [qctl 0.8.0-rc.1][qctl-rc1] from [release run 46036007862][qctl-release-run].
    The `linux-arm64` job failed once on a cache-key race and passed on rerun (attempt 2); [verctl#434][verctl#434] fixed the cause.

[qctl#200]: https://github.com/victor-software-house/qctl/pull/200
[qctl-merge]: https://github.com/victor-software-house/qctl/commit/b4ff7f85fe3b0aa5464f993638f24c1de43586c3
[qctl-head]: https://github.com/victor-software-house/qctl/commit/61f341cc798176baccb64195372d62a1e43dda5d
[qctl-ci]: https://github.com/victor-software-house/qctl/actions/runs/46034379679
[qctl-pin]: https://github.com/victor-software-house/qctl/commit/776a9554dfbf05fccd8be052a550efa4146f1f38
[qctl-tests]: https://github.com/victor-software-house/qctl/commit/e9e68859b2723522915067295c9bcac17c45228d
[qctl-golden]: https://github.com/victor-software-house/qctl/commit/0b415f5c4b060cc71eeba1ab99159dc8eae19f1d
[qctl-docs]: https://github.com/victor-software-house/qctl/commit/1bd2590b66b6f8076125e888034d2ba8b1759881
[qctl-release-commit]: https://github.com/victor-software-house/qctl/commit/aa012fe61f80661db1e5db2d73e31a8ec7fb479a
[qctl-cargo]: https://github.com/victor-software-house/qctl/blob/b4ff7f85fe3b0aa5464f993638f24c1de43586c3/Cargo.toml
[qctl-lock]: https://github.com/victor-software-house/qctl/blob/b4ff7f85fe3b0aa5464f993638f24c1de43586c3/Cargo.lock
[qctl-skill]: https://github.com/victor-software-house/qctl/blob/b4ff7f85fe3b0aa5464f993638f24c1de43586c3/skills/qctl/SKILL.md
[qctl-instructions]: https://github.com/victor-software-house/qctl/blob/b4ff7f85fe3b0aa5464f993638f24c1de43586c3/src/instructions.md
[qctl-ver]: https://github.com/victor-software-house/qctl/blob/b4ff7f85fe3b0aa5464f993638f24c1de43586c3/.ctl/ver.yaml
[qctl-changeset]: https://github.com/victor-software-house/qctl/blob/aa012fe61f80661db1e5db2d73e31a8ec7fb479a/.changeset/ctl-core-0-8.md
[qctl-rc1]: https://github.com/victor-software-house/qctl/releases/tag/v0.8.0-rc.1
[qctl-release-run]: https://github.com/victor-software-house/qctl/actions/runs/46036007862
[verctl#434]: https://github.com/victor-software-house/verctl/pull/434
