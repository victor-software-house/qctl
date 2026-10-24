# qctl

Rust policy CLI for in-repo `tasks.yaml` work queues.

- Operator contract: `qctl instructions` and `skills/qctl/SKILL.md`.
- Templates under `.ctl/operator/` keep domain prose and import ctl-core's version, mounted invocation/no-`--`, and command-inventory fragments from the Clap-derived `Surface`. `operator_docs` byte-compares both committed renders; update them with `UPDATE_OPERATOR_DOCS=1 cargo test operator_docs`.
- This repo's queue is [`tasks.yaml`](tasks.yaml) (`QCTL-###`).
- `horizon` maps research/evaluations that are not startable. Do not put
  them on `queue` and do not set `active` to a horizon id.
- Queue, horizon, and archive partition one global task corpus. IDs are
  continuous from `PREFIX-001` through the maximum and appear exactly once;
  status changes move one row rather than copying it between lists.
- Acceptance states observable, demonstrably true end conditions for closing a
  row. It does not carry implementation instructions or procedural steps;
  `plan` and `notes` own how the work is done.
- `notes` is an ordered list, and each `-n/--note` is one item. Every list
  item goes through one writer, `Document::push_items`, in the layout
  `style.list_items` declares (`line` by default) and the quotes
  `style.quotes` declares. Under `line` a verb refuses a note holding a line
  break, and `fmt` writes older block items on one line. Never insert
  mechanical sentence wraps.
- Schema is types + schemars (QCTL-001). Generated JSON lives only here.
  Consumers pin a `$schema` URL and run `qctl check`. `schema_version` is 4:
  `notes` is a list, a ledger declares its `style`, and `completed` is a moment
  in the zone that block names. A schema 3 file is rewritten by `qctl fmt`.
  A rule that both `check` and the verbs must agree on is stated
  once, as a `#[garde(...)]` attribute — schemars reads those, so a second
  `#[schemars(...)]` copy is drift waiting to happen.
- Mutations rewrite only the lines they change (QCTL-002). Each scenario in
  `tests/fixtures/mutations/` records what the verb wrote as one marked-up
  file: `-` removed, `+` added. When a change to that output is intended,
  `mise run snapshots` shows it scenario by scenario and accepts it.
- `qctl fmt` applies the declared style; `qctl fmt --check` is the hook form.
  New style options are additive: a field with a default equal to today's
  behaviour is a minor change, not a migration.
- The repository setup comes from the shared Copier template (project
  `rust-crate`, layout `crate`): `.copier-answers.yml` records its tag and
  answers. Take a new tag with `copier update --defaults` and check
  `git diff --check` before committing. `ci.yml` and `template.yml` call the
  shared workflows of the public actions repository at the commit the
  template pins; their checks report as `verify / Verify` and
  `template / Template`. The drift check fails only a pull request that
  changes `.copier-answers.yml` without reaching the newest tag; a push to
  `main` or the daily run keeps a "Template update available" issue open
  while a newer tag would change a file. The shared Verify sets git's default
  branch to `main` for the tests that run `git init`. These local edits
  survive updates: `lefthook.yml` keeps the push-range commit lint,
  `.rumdl.toml` allows the README banner's HTML, and
  `.typos.toml` ignores commit hashes cited in prose.
- `mise run verify` is the gate, composed in `mise.toml` from the actions
  repository's `tasks/rust.toml` (included by `git::` URL): toolchain pins,
  rustfmt on the dated nightly, clippy with the `Cargo.toml` lints denied,
  `check:features`, `cargo test`, doc-tests, `doc` (rustdoc as docs.rs builds
  it), cargo-deny licenses, bans, and sources, cargo-machete,
  `check:versions`, and `check:repo`
  (taplo, rumdl, typos, actionlint, OpenSpec validation, and `check:local`,
  which holds this repository's own checks).
  qctl's own tasks live in `mise-tasks/`. CI adds `deny:advisories` and
  `semver`, which need the network. `mise.toml` pins stable 1.99.0 with
  `mr_boxington = true`, locally and in CI, where `jdx/mr-boxington-action`
  restores its cache; `rust-toolchain.toml` and `rust-version` match.
- The pull request's CI runs `verify`; no hook runs it. Lefthook: pre-commit
  checks formatting and scans the staged diff for credentials, commit-msg
  lints the message with committed (`committed.toml`), and pre-push runs
  `mise install --locked`, the commit lint over every pushed commit, and a
  full-history credential scan. Every hook runs with `MISE_ENV=dev,hooks`.
  A fresh clone arms them with `mise install`, through
  `mise.dev.toml`'s postinstall hook; CI never loads `dev`, so no runner gets
  them.
- `.github/workflows/release.yml` calls verctl's reusable release workflow on
  every push to `main`: it publishes each version commit not yet fully
  released to crates.io, builds the tarballs (the linux-x64 build attaches
  `schema/tasks.schema.json`, task `release:qctl` in `mise.release.toml`), and
  opens the Version PR on `verctl/release/main` with auto-merge. Bump its pin
  with the verctl pin in `mise.release.toml` and the `ver` include in
  `mise.toml`.
- This repo's release declarations live in `.ctl/ver.yaml`; `.ctl/` is the
  directory every ctl CLI shares, and qctl's own project config will land
  beside it as `.ctl/q.yaml`. There is no `verctl.toml`.
- Served files — `tasks/q/q` and `examples/mise.toml` — are rendered from
  `.ctl/templates/`. README's `?ref=` and install line plus the bundled skill's
  `version:` are declared under `patterns` in `.ctl/ver.yaml` (QCTL-009,
  QCTL-022). Edit the template or the pattern, never the rendered file; the
  Version PR rewrites every site onto the commit the tag names. A template git does not track renders nowhere.
  The `q` task carries `ctl_core::mount_directives("q")`: `raw`, `raw_args`,
  and the `#USAGE mount` line, so piped output stays what qctl wrote.
  Put it in the template. Do not copy it onto `tasks/q/q` while that
  file still pins a release that lacks `--usage-spec`. The Version PR
  writes the mount and the new pin together. Operators run
  `mise run q status` with no `--`.
- A consumer wires one pin, not two: the `?ref=` tag on the task include. The
  served task carries its own `#MISE tools` version and wins on PATH, so a
  consumer `[tools]` entry can only disagree with it — and needs a lockfile
  entry to survive `mise install --locked` (ctl-core#9 hit exactly that).
- `qctl hook install` never edits `lefthook.yml`. Lefthook has no API to add
  a command. If that file exists, install prints the `mise run q close-from-git`
  snippet and exits non-zero until Lefthook already runs it. Otherwise it
  writes `.git/hooks/pre-push`. `--force` overwrites the git hook only.

## Strings

Multiline Rust is `indoc!` / `formatdoc!` / `writedoc!` / `printdoc!` /
`eprintdoc!` / `concatdoc!`. **No `concat!`.** **No escaped `\n` in a
document.** Leave a raw `\n` only when that *is* the test (a single `'\n'`
char, a one-line protocol payload, CRLF). This covers production code and
test fixtures and assertions.

## Check Behavior

- `qctl check` does not skip a git trailer scan that failed. A scratch
  ledger that is not in the current repository needs `--no-git`.

## Ownership and Design

- `src/cli.rs` is the Clap grammar. `src/report.rs` owns serializable command
  results. Domain modules return those results and never print.
- Growing domain features use nested module trees, not flat helper files.
  `src/edit.rs` is the row-edit entry point; its field, list, policy, and
  position mechanics live under `src/edit/`. Document rendering helpers live
  under `src/document/`, and dependency ordering under `src/ledger/`.
  Cross-feature APIs use the narrowest visibility that works.
- Queue dependency validation builds one id-to-position index and is shared by
  `add` and `edit`; do not restore per-blocker queue scans.
- `src/presentation.rs` maps reports onto ctl-core semantic documents. ctl-core
  alone owns help, pretty/colorless/JSON emission, stream selection, quiet
  behavior, terminal width, styling, and tables.
- A ledger is parsed through ctl-core's declared input (`ctl_core::input`),
  which reports every problem with its file and line. It is the one ctl-core
  path a domain module may use; `tests/architecture.rs` refuses the rest.
- `-f` remains qctl's ledger-file shorthand. The root composes ctl-core
  `FormatLong` with `ColorLong`, so shared `--format` never takes `-f` back.
- The root parser carries `#[command(env_prefix = "QCTL")]` (ctl shared
  rule 7), so every long option reads `QCTL_<LONG>`. The ledger path is
  `--file`, then `QCTL_FILE`, then `tasks.yaml`; `TASKS_LEDGER` is not read.
  The test helper `common::binary` clears every `QCTL_*` variable.

## Changes

Plan a behaviour or contract change as an OpenSpec change in
`openspec/changes/<name>/` before writing code. `openspec/config.yaml` holds
this repository's context and rules, and `openspec validate <name>` checks the
change. A `tasks.yaml` row names its change through its `plan:` path; one change
may serve several rows, and a row names at most one change.
