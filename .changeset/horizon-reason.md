---
"qctl": patch
---

A horizon row now says why it waits in `reason` instead of `kind`, beside `open`, which names what would end the wait. The values stay `research`, `evaluation`, and `deferred`.
`-r/--reason` replaces `-k/--kind` on `add --horizon`, `park`, and `edit`, so the variable `QCTL_KIND` becomes `QCTL_REASON`. JSON output, the `status` horizon table, and error messages say `reason`.
`schema_version` is now 6. `qctl fmt` upgrades a schema 5 ledger in place, renaming each horizon row's `kind` key to `reason` and keeping its value, quotes, and comments; it upgrades schema 3 and 4 the same way. Other verbs refuse schema 5 until `qctl fmt` runs.
