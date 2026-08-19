---
qctl: patch
---

State the ledger contract once, as Rust types. `schema/tasks.schema.json` is
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
