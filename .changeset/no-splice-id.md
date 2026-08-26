---
qctl: patch
---

Tell agents not to splice a new `- id:` into `tasks.yaml`. `add` and `park` create rows; the other verbs still beat a hand edit because `archive` also clears `blocked_by`. State that notes and acceptance on an existing row have no verb yet.
