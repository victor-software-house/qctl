# qctl

Control in-repo YAML work queues. One file, one active task, file order is
priority. Replaces copied Ajv `test:ledger` scripts.

```sh
mise run q -- check
mise run q -- status
mise run q -- add -t 'Title' -s repo -o 'Done when…' -a 'Acceptance'
mise run q -- start OMX-001
mise run q -- archive OMX-001 -e 'Shipped.'
```

Consumer mise catalog (same shape as forkctl 0.0.21):

```toml
[task_config]
includes = [
  "git::https://github.com/victor-software-house/qctl.git//tasks/q?ref=<immutable-ref>",
  "mise-tasks",
]
```

Do not name this `taskctl` (existing Go Make alternative) or `pi-tasks`
(different VSH TypeScript product).
