---
qctl: minor
---

An archived row now says when it left the queue, not only on which day.
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
