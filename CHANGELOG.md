# Changelog

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

