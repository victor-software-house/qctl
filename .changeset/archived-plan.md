---
qctl: patch
verctl:group: fixed
---

`qctl edit ID -l PATH` now changes the plan of an archived row, and `-U plan` removes it, so archiving an OpenSpec change after its row needs no hand edit of `tasks.yaml`. An archived row still refuses every other field but notes, evidence, and links.
