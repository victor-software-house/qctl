# qctl

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/banner-dark.svg">
  <img src="docs/banner.svg" alt="qctl — In-repo YAML work queues. One file, one active task.">
</picture>

Control in-repo YAML work queues. One file, one active task, file order is
priority. Queue, horizon, and archive partition one continuous, globally unique
id sequence. Replaces copied Ajv `test:ledger` scripts.

Every command returns typed data through ctl-core. Human output is pretty by
default; `--color never` or `--no-color` keeps the same layout without ANSI,
`--format json` emits the report as one JSON line, and `--quiet` suppresses
successful human output only. `-f` remains the ledger-file shorthand.

```sh
mise run q check
mise run q status
mise run q status --format json
mise run q check --format json
mise run q add -t 'Document the schema' -s docs -o 'Readers can verify every row field.' -a 'The schema page names the queue, archive, and horizon fields.'
mise run q start OMX-001
mise run q archive OMX-001 -e 'Shipped.'
```

Acceptance states observable, demonstrably true end conditions for closing the
task. Write the resulting state in the present tense. Put implementation steps
in `plan` and supporting context in `notes`. Repeat `-n/--note` for distinct
facts; each argument is one ordered list item, and intentional line and paragraph
breaks survive YAML readback.

Consumer mise catalog — copy [`examples/mise.toml`](examples/mise.toml), which
pins the tool and the task include to the same release:

```toml
[task_config]
includes = [
  "git::https://github.com/victor-software-house/qctl.git//tasks/q?ref=v0.4.1",
  "mise-tasks",
]
```

Install:

```sh
cargo install qctl --locked
# or the native tarball:
mise x github:victor-software-house/qctl@0.4.1 -- qctl --version
```

Do not name this `taskctl` (existing Go Make alternative) or `pi-tasks`
(different VSH TypeScript product).
