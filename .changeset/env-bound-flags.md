---
"qctl": patch
---

Every long option now reads `QCTL_<LONG>`, as in the other ctl CLIs: `QCTL_FILE` names the ledger, `QCTL_FORMAT=json` asks for JSON, and `--help` prints each variable. A flag on the command line wins over its variable.
`TASKS_LEDGER` is no longer read. A caller that set it sets `QCTL_FILE` instead.
