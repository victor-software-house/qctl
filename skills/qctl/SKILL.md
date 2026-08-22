---
name: qctl
description: >-
  Operate qctl in-repo YAML work queues: status, check, add, start, archive,
  park, promote, show, and instructions. Use when a repository has tasks.yaml, prefix QCTL
  or another qctl prefix, mise run q, horizon research/evaluation rows, or
  the user mentions qctl, the work queue, or replacing Ajv test:ledger.
  Do not use the vault ompex/task-ledger plugin.
license: MIT
compatibility: Requires a qctl binary. A repository-mounted mise `q` task may provision an exact version after a GitHub Release exists.
---

# qctl

Treat `qctl` as the policy owner for `tasks.yaml`. Do not reimplement
validation with a copied Ajv test. Do not vendor `tasks.schema.json`.

`qctl instructions` and `--help` are the installed-version contract. This
skill is the operator workflow; it does not duplicate the flag grammar.

## Establish the contract

1. Find `tasks.yaml`. Read `active`, then `queue`, then `horizon` if the
   ask is research, evaluation, or "not now".
2. Invoke `mise run q <args>` when the repo mounts the catalog.
   Otherwise `qctl`. Never invent commands. Never `mise run q --`.
3. Run `status`, then `check`, before mutation.

```sh
mise run q status
mise run q check
mise run q instructions
```

## Three lists

- **queue** — short-term. File order is priority. `active` is `queue[0]` or `null`.
- **archive** — done or dropped. Newest first. IDs never reused.
- **horizon** — mapped work with no start condition (`research`, `evaluation`,
  `deferred`). `open` names the missing condition. Not priority. Never `active`.

Promote horizon → queue with `qctl promote ID -a …` after `open` is
resolved. Do not start a horizon id.

## Mutate

```sh
qctl add -t 'Title' -s repo -o 'Done when…' -a 'Acceptance'
qctl add -t 'Title' -s repo -o 'Done when…' -a 'Acceptance' --notes 'Why' --blocked-by QCTL-001 --after QCTL-001
qctl add -t 'Title' -s repo -o 'Done when…' --horizon --kind research --open 'The missing fact'
qctl park -t 'Title' -s repo -o 'Done when…' --kind research --open 'The missing fact'
qctl promote QCTL-001 -a 'Acceptance'
qctl start QCTL-001
qctl archive QCTL-001 -e 'Shipped.'
qctl close-from-git
qctl hook install
```

`--plan` must be a file next to the ledger. `add --horizon` and `park`
write `horizon:` if the ledger omitted it. `close-from-git` archives
queued ids closed by `Closes PREFIX-NNN` / `Completes: PREFIX-NNN` in
the commit **body**. `hook install` adds `mise run q close-from-git` to
Lefthook when `lefthook.yml` exists; otherwise a git pre-push. Neither
amends.

Optional `patch:` is a forkctl patch name. When `active` has one, select
that patch before editing.

`$schema` is a **pinned** qctl URL. `init` must not copy a schema file into
the consumer.

## Stop conditions

Stop and ask when the work has no start condition (horizon, do not queue),
when `active` would point at horizon, or when installed help disagrees with
remembered syntax.
