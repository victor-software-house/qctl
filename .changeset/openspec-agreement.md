---
"qctl": minor
---

Beside an `openspec/` folder, `qctl check` holds each row to the OpenSpec change its `plan:` names: it reports a finished row whose change is still open, a plan pointing into a change since archived (naming the archive folder), and an open change no row names.
`qctl archive` refuses the last row of an open change unless `-K/--keep-change-open REASON` says why it stays open, and `close-from-git` leaves such a row queued.
The new `qctl close-from-openspec` archives the queued rows of an archived change, with the archive folder as evidence, and points every row that named it at the moved file.
`schema_version` is now 5, because an archived row may carry `keep_change_open`: `qctl fmt` upgrades a schema 4 ledger by that one line, and other verbs refuse schema 4 until it runs.
