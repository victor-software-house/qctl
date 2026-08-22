---
qctl: patch
---

`add --horizon` and `park` write a `horizon:` key when the ledger omitted it. `--plan` is checked the same way `check` is. A tail add to an empty queue no longer pretends the caller passed `--before`.
