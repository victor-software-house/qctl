---
qctl: patch
---

A mutation now rewrites the lines it changes and copies the rest. `add`, `start`
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
