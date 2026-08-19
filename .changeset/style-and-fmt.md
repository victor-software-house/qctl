---
qctl: minor
---

A ledger now declares how it is written, and `qctl fmt` writes it that way.

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
