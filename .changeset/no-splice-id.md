---
qctl: patch
---

Existing rows can now be revised without splicing YAML: `edit ID` updates fields, list items, and dependency-safe queue position, while `park ID` demotes queued work to the horizon. Notes are ordered list items in schema 4, and `fmt` upgrades schema 3 scalar notes without rewriting unrelated rows.
