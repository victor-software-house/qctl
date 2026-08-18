---
qctl: patch
---

Keep Cargo.lock in step with the bump. The Version PR now regenerates it and
commits it, so `cargo package --locked` no longer fails Verify on a dirty tree.
