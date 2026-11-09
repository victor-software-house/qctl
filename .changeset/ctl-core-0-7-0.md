---
qctl: patch
---

qctl builds on ctl-core 0.7.0, which renders through its own terminal engine and links one serde-saphyr with yamled 0.0.12.
The help of `--color`, `--no-color`, and the `pretty` format takes ctl-core's new wording, and a verb missing a required argument prints its help on stderr with exit 2.
