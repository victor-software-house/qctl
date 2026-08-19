---
qctl: patch
---

`mise run q` now runs the qctl its own pin names. The served task called
`mise where github:victor-software-house/qctl`, which resolves the version from
the surrounding config — not from the task's `#MISE tools` line — so on a machine
holding a newer qctl for another repo, a consumer pinned to an older release
silently ran the newer binary. mise already puts the task's pinned tool first on
PATH, so the task execs `qctl` and the pin decides.

Measured on one machine with 0.0.1, 0.1.0 and 0.1.1 installed, same task pinned
to 0.0.1: the new form runs `qctl 0.0.1`, the old form runs `qctl 0.1.1`.
