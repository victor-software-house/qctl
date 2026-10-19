# Changelog

## qctl 0.5.11

- The shared verify runs verctl's version guard: ctl-core's Rust tasks at 0.6.27 and verctl at 0.5.1, so a manifest version changes only on the Version PR ([#140](https://github.com/victor-software-house/qctl/pull/140)).

## qctl 0.5.10

- Releases run through verctl's reusable release workflow at 0.5.0: each push to `main` publishes any version commit not yet fully released, builds its tarballs with the ledger schema attached, and opens or updates the Version PR on `verctl/release/main` ([#138](https://github.com/victor-software-house/qctl/pull/138)).

## qctl 0.5.9

- A ledger's `style` declares `list_items` (`line` or `auto`) and `quotes` (`auto`, `single`, or `double`).
The defaults are `line` and `auto`: every list item a verb writes sits on one line, plain when it reads back unquoted, and `add` and `edit` refuse a `-n/--note` that holds a line break.
`qctl fmt` writes older folded and literal list items on one line, splits a multi-line note into one item per line with its line breaks and blank lines dropped, and re-quotes every one-line string outside `style` to the declared quotes; every other value reads back the same.
A ledger written before this release changes shape on its first `qctl fmt`; `list_items: auto` keeps block items and multi-line notes ([#118](https://github.com/victor-software-house/qctl/pull/118)).

## qctl 0.5.8

- Every long option now reads `QCTL_<LONG>`, as in the other ctl CLIs: `QCTL_FILE` names the ledger, `QCTL_FORMAT=json` asks for JSON, and `--help` prints each variable. A flag on the command line wins over its variable.
`TASKS_LEDGER` is no longer read. A caller that set it sets `QCTL_FILE` instead ([#129](https://github.com/victor-software-house/qctl/pull/129)).

## qctl 0.5.7

- qctl builds on ctl-core 0.6.23 and yamled 0.0.7, and reads YAML through serde-saphyr 2.0.0.
A list item that needs quotes and holds an apostrophe is now written in double quotes, `"The operator's call: yes."`, instead of doubling the apostrophe inside single quotes ([#127](https://github.com/victor-software-house/qctl/pull/127)).

## qctl 0.5.6

- The docs.rs page builds with every feature, and every public function that can fail says when it fails in an `# Errors` section ([#121](https://github.com/victor-software-house/qctl/pull/121)).
- Every public item is documented and implements `Debug`.
`--help` now describes every flag and argument, including `--title`, `--scope`, `--outcome`, `--patch`, `--plan`, `--kind`, `--open`, and the task id.
The `--disposition` and `--unset` values say what each one means.
The generated schema gives each `section_order` value a description, as it already did for the other enumerations ([#122](https://github.com/victor-software-house/qctl/pull/122)).

## qctl 0.5.5

- Fix the three documentation links that failed the docs.rs build since 0.4.7: two pointed at private items and one at a function that no longer exists.

The `--prefix` help and the prefix documentation now give public example prefixes ([#119](https://github.com/victor-software-house/qctl/pull/119)).
- The operator instructions and bundled skill now match the commands: `archive -d dropped` is documented, an archived row's plan is editable, and the skill no longer points at an unrelated plugin ([#120](https://github.com/victor-software-house/qctl/pull/120)).

## qctl 0.5.4

### Fixed

- `qctl edit ID -l PATH` now changes the plan of an archived row, and `-U plan` removes it, so archiving an OpenSpec change after its row needs no hand edit of `tasks.yaml`. An archived row still refuses every other field but notes, evidence, and links ([#112](https://github.com/victor-software-house/qctl/pull/112)).

## qctl 0.5.3

### Fixed

- The served `q` mise task now runs `raw`. mise relayed its output and appended an SGR reset to every line whenever mise's own colour was on, as under `CLICOLOR_FORCE=1`, so `mise run q status --format json | jq` received an ESC byte qctl never wrote ([#110](https://github.com/victor-software-house/qctl/pull/110)).

## qctl 0.5.2

### Fixed

- A parse error under `--format json` is now a JSON error even when the bad argument comes before the flag, as in `qctl status --bogus --format json`. It printed as text, coloured when `CLICOLOR_FORCE` or `FORCE_COLOR` was set, to a caller that asked for JSON. JSON output stays plain whatever the colour environment says ([#106](https://github.com/victor-software-house/qctl/pull/106)).

## qctl 0.5.1

- Raise `rust-version` to 1.99.0, the pinned stable the shared template builds, lints, and tests on. A build with an older compiler stays on the previous release.

## qctl 0.5.0

- A ledger can declare `style: {row_spacing: blank_line}`, and `qctl fmt` then keeps exactly one blank line between rows: a lost separator is put back, and `fmt --check` reports it. The first row stays tight to its list's key, the blank line goes above a row's comments, and a row ending in a `|+` or `>+` block is left alone. The default, `as_written`, changes nothing.
- Render the bundled skill and installed instructions from qctl's Clap-derived ctl-core Surface. The shared version, mounted invocation, no-`--` rule, and visible command inventory now fail byte-exact tests when the CLI grammar or committed documents drift.

## qctl 0.4.7

- Trailer scans read the ledger's own repository when qctl runs inside another repository's git hook. Git exports the hook repository's `GIT_DIR` and related variables, and qctl's git calls inherited them.

## qctl 0.4.6

- Read ledgers through ctl-core 0.6.9 and edit them through yamled 0.0.5. A parse error in a value reached through an alias names where that value is defined, and an edit refused by a kept tag names the tag.

## qctl 0.4.5

- Edit ledgers through yamled. `fmt` now reorders lists around a comment that belongs to none of them, leaving it in place, instead of refusing; a moved row keeps its list's own spacing; and literal and folded notes are written as `|-` and `>-`.
- Read every ledger through ctl-core's declared input: a value or parse problem, and each schema error from `qctl check`, now names its file and line.

## qctl 0.4.4

- Let the environment pick each style the binary leaves open: `CTL_CORE_RECORD_STYLE`, `CTL_CORE_LIST_STYLE`, `CTL_CORE_ROW_SEPARATION`, and `CTL_CORE_JSON_LAYOUT`. An unaccepted value is ignored with a warning that names the accepted values.

## qctl 0.4.3

- Adopt ctl-core 0.6.3: borderless records, bold row ids, indented JSON, and an 80-column fallback width in piped output.

## qctl 0.4.2

- Adopt ctl-core's isolated Surface model feature so short-option contract tests no longer enable the MiniJinja template engine.
- Remove disposable Cargo package staging before rust-cache cleanup so successful publish runs no longer emit false missing-directory annotations.

## qctl 0.4.1

- Adopt ctl-core 0.5.1 so automatically sized output reserves its configurable nested-shell margin, while the shared surface contract prevents new long-only flags.
- Make `qctl check` treat queue, horizon, and archive as one task corpus, reporting missing id numbers, duplicate rows, and ids claimed by more than one status.
- Existing rows can now be revised without splicing YAML: `edit ID` updates fields, list items, and dependency-safe queue position, while `park ID` demotes queued work to the horizon. Notes are ordered list items in schema 4, and `fmt` upgrades schema 3 scalar notes without rewriting unrelated rows.
- Clarify that acceptance criteria are observable end conditions that must be demonstrably true before a task closes, while plans and notes describe how the work is done.
- Write each repeated note as a distinct ordered list item, preserving intentional line and paragraph breaks while using readable block scalars for YAML-sensitive prose.

## qctl 0.4.0

- Route every command through ctl-core's typed App and semantic Document presentation. Add shared pretty, colorless, JSON, quiet, stream, and error behavior; `status --format json` exposes ledger state and `check --format json` exposes its problem array.

## qctl 0.3.1

- ctl-core 0.1.1. `features = ["usage"]` compiles without `help`.

## qctl 0.3.0

- `qctl close-from-git` archives queued ids closed by commit-body trailers. `qctl hook install` writes a git pre-push, or prints the Lefthook snippet to add; it never edits `lefthook.yml`. Neither amends.
- `qctl check` reports a git trailer scan that failed or that skipped because the ledger is not in the current repository. It no longer passes by returning no errors.
- Served `q` task mounts the ctl-core Usage spec. `mise run q status` needs no `--`.

## qctl 0.2.2

- `add --horizon` and `park` write a `horizon:` key when the ledger omitted it. `--plan` is checked the same way `check` is. A tail add to an empty queue no longer pretends the caller passed `--before`.

## qctl 0.2.1

- Pin Version PR, publish, and asset at verctl 0.2.1 so a release tags the published commit.

## qctl 0.2.0

- `add` writes notes, blockers, plan, and links, and can place the row with `--before` / `--after` or write a horizon row with `--horizon`.
- `park` writes a horizon row. `promote` moves it onto the queue tail with acceptance, dropping kind and open, and leaves `active` alone.

## qctl 0.1.2

- `mise run q` now runs the qctl its own pin names. The served task called
`mise where github:victor-software-house/qctl`, which resolves the version from
the surrounding config — not from the task's `#MISE tools` line — so on a machine
holding a newer qctl for another repo, a consumer pinned to an older release
silently ran the newer binary. mise already puts the task's pinned tool first on
PATH, so the task execs `qctl` and the pin decides.

Measured on one machine with 0.0.1, 0.1.0 and 0.1.1 installed, same task pinned
to 0.0.1: the new form runs `qctl 0.0.1`, the old form runs `qctl 0.1.1`.

## qctl 0.1.1

- Every version a consumer reads out of a tag now comes from the release that tag
names. `tasks/q/q` and `examples/mise.toml` are rendered from
`.verctl/templates/`, and README's two spellings — the `?ref=` include and the
`mise x …@` install line — are declared as `[patterns]`, so the Version PR
rewrites all five sites on the commit the tag points at.

Three tags shipped a task file that installed the previous release: `git show
v0.1.0:tasks/q/q` pinned `0.0.1`, because every site was a hand bump somebody
had to remember. A rendered file cannot drift, and a declared pattern that stops
matching stops the release instead of shipping quietly.

`examples/mise.toml` also gained the `[tools]` entry it was missing, so copying
it installs the pinned binary rather than whatever the task include happens to
carry.

## qctl 0.1.0

- An archived row now says when it left the queue, not only on which day.
`completed` is a moment to the second: `2026-08-18T22:46:12`. A day on its own
was not enough to tell two rows archived in the same session apart, or to order
them.

The shape is stated once and both readers of the contract enforce it — `check`
through a pattern in the generated schema, the verbs through the same compiled
pattern. Before, a `date-time` format alone would have accepted a stamp from
`check` that the verbs refused.

Two more things went with it. `archive` takes the archived id out of every
`blocked_by` that named it, so the ledger it leaves behind still passes `check`
instead of failing with "is not queued". And `archive` loads the ledger before
editing it, as `add` and `start` already did, so no verb writes to a file whose
values it has not vouched for.
- Archiving a row from between two others no longer takes their separator with it.

A row's span reaches to where the next row begins, so it already held the blank
line between them; taking the blank line above as well removed two separators for
one row, and the rows either side ended up written against each other. The last
row of a list has no next row, so there the blank above is still the one to take.

Found by reading this repo's own ledger: two rows had lost their separator, and no
fixture archived from the middle of a list. One does now.
- State the ledger contract once, as Rust types. `schema/tasks.schema.json` is
now generated from them by `qctl schema`, and a test fails when the committed
file is not what the types say — so the schema an editor reads and the binary
that enforces it cannot drift apart.

The contract also gained the fields it always had on paper: every row kind
carries `links` and `notes`, an archived row carries `scope`, `outcome`,
`evidence` and `disposition`, and `disposition` and `kind` are real
enumerations rather than free strings. `qctl` now rejects a bad value where it
reads the file, naming the field: an id that is not an id, an empty title, a
row with nothing to accept, a `completed` that is not a date, a plan pointing
outside the repository.

One behavior change worth knowing: a malformed id is refused when the ledger
loads, so it no longer arrives at the graph checks as `does not match
PREFIX-NNN`. A well-formed id belonging to another repository still reports
there, because only this ledger knows its own prefix.
- A mutation now rewrites the lines it changes and copies the rest. `add`, `start`
and `archive` used to parse the ledger into values and serialize a new file from
them, which kept every value and lost everything else: one `qctl start` against
a 479-line ledger changed all of it. The same move now changes 67 lines — the
row that moved, and the `active` line.

What survives, because it is never rewritten: the `# yaml-language-server:`
header, comments above a row and beside a key, blank lines between rows, folded
`>-` scalars at the width they had, quote style, inline `[…]` lists, and the
indentation the file already used.

Two defects went with it. `archive` now takes the archived id out of every
`blocked_by` that named it, so the ledger it leaves behind still passes `check`
instead of failing with "is not queued". And `archive` loads the ledger before
editing it, as `add` and `start` already did, so no verb writes to a file whose
values it has not vouched for.

Structure comes from a real parser rather than from string matching: `yamlpath`
for spans, `yamlpatch` for key edits, and the ledger types for any value that
has to be written. Two things the format leaves open are decided here and
documented in `src/document.rs`: a row moves as a byte range, and a comment
written directly above a row belongs to that row.
- A ledger now declares how it is written, and `qctl fmt` writes it that way.

`schema_version` is **3**. The `style` block is new and entirely optional —
every option defaults to what qctl already wrote, so a file with no `style`
block is valid:

| Option | Default | What it decides |
|:--|:--|:--|
| `timezone` | `+00:00` | The offset `completed` is written in |
| `section_order` | `[queue, archive, horizon]` | The order the three lists appear in |
| `indent` | `2` | How far a row sits under its list's key |
| `archive_order` | `newest_first` | Whether `fmt` sorts the archive |
| `normalize_on_write` | `false` | Whether every verb normalizes the whole file |

`completed` no longer carries an offset, because the ledger carries one:
`2026-08-18T22:46:12` rather than `2026-08-19T01:46:12Z`. To migrate a ledger,
shift each stamp into the zone you declare, drop the `Z`, and set
`schema_version: 3`.

`qctl fmt` rewrites a ledger in its declared style. `qctl fmt --check` writes
nothing, names the lines that differ, and exits non-zero.

`fmt` does not guess. It changes what an option names, sorts the archive when
asked, and removes whitespace nobody chose — a space at the end of a line, a
second blank line, a blank line between a list's key and its first row, a file
that does not end in exactly one newline. It never adds a blank line, a comment
or a key, and it never rewrites a value.

`normalize_on_write` is off, so a verb still rewrites only the lines it changes
and a diff shows the work and nothing else. Turn it on and every verb leaves the
file fully normalized instead.

Changing `timezone` does not move the stamps already written: nothing records
which zone an old stamp was taken in, so nothing can convert it without moving a
moment.

## qctl 0.0.2

- Release through verctl: CI on every PR, a Version PR on main, and
`darwin-arm64` plus `linux-x64` tarballs on the GitHub Release.
- Keep Cargo.lock in step with the bump. The Version PR now regenerates it and
commits it, so `cargo package --locked` no longer fails Verify on a dirty tree.

