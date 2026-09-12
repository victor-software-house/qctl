# qctl

Rust policy CLI for in-repo `tasks.yaml` work queues.

- Operator contract: `qctl instructions` and `skills/qctl/SKILL.md`.
- This repo's queue is [`tasks.yaml`](tasks.yaml) (`QCTL-###`).
- `horizon` maps research/evaluations that are not startable. Do not put
  them on `queue` and do not set `active` to a horizon id.
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
- This repo's release declarations live in `.ctl/ver.yaml`; `.ctl/` is the
  directory every ctl CLI shares, and qctl's own project config will land
  beside it as `.ctl/q.yaml`. There is no `verctl.toml`.
- Served files — `tasks/q/q` and `examples/mise.toml` — are rendered from
  `.ctl/templates/`. README's `?ref=` and install line plus the bundled skill's
  `version:` are declared under `patterns` in `.ctl/ver.yaml` (QCTL-009,
  QCTL-022). Edit the template or the pattern, never the rendered file; the
  Version PR rewrites every site onto the commit the tag names. A template git does not track renders nowhere.
  The `q` task's `#USAGE mount` line is `ctl_core::mount_line("q")`.
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
- `-f` remains qctl's ledger-file shorthand. The root composes ctl-core
  `FormatLong` with `ColorLong`, so shared `--format` never takes `-f` back.
