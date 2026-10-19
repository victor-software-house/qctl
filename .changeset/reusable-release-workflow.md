---
qctl: patch
---

Releases run through verctl's reusable release workflow at 0.5.0: each push to `main` publishes any version commit not yet fully released, builds its tarballs with the ledger schema attached, and opens or updates the Version PR on `verctl/release/main`.
