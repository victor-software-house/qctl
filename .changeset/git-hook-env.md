---
qctl: patch
---

Trailer scans read the ledger's own repository when qctl runs inside another repository's git hook. Git exports the hook repository's `GIT_DIR` and related variables, and qctl's git calls inherited them.
