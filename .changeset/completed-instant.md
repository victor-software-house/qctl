---
qctl: minor
---

An archived row now says when it left the queue, not only on which day.
`completed` is an instant to the second, in UTC, in exactly one shape:
`2026-08-19T02:00:41Z`. A day on its own was not enough to tell two rows
archived in the same session apart, or to order them.

`schema_version` is **2**. A ledger still declaring 1 is refused, because its
`completed` values cannot be read as instants. To migrate a ledger, replace each
archived row's day with the instant it happened — the commit that first shows
that row under `archive:` is the evidence — and set `schema_version: 2`.

The shape is stated once and both readers of the contract enforce it: `check`
through a pattern in the generated schema, the verbs through the same compiled
pattern. Before, `format: date-time` alone would have accepted
`2026-08-19T00:20:19-03:00` from `check` while the verbs refused it.

Stored stamps are UTC so they sort wherever they are read. `qctl show` prints
them where the work happens, three hours behind.
