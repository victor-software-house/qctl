---
name: qctl
description: >-
  Operate qctl in-repo YAML work queues: status, check, add, start, archive,
  park, promote, edit, show, fmt, and instructions. Use when a repository has
  tasks.yaml, prefix QCTL or another qctl prefix, mise run q, horizon
  research/evaluation rows, or the user mentions qctl, the work queue, or
  replacing Ajv test:ledger. Use when adding, parking, or editing a ledger
  row instead of editing tasks.yaml by hand.
  Do not use the vault ompex/task-ledger plugin.
license: MIT
version: 0.4.0
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
mise run q check --no-git
mise run q instructions
```

## Read output

Human output is pretty by default. Use `--color never` or `--no-color` for
the same layout without ANSI. Use `--format json` for one typed JSON report
on stdout; `status` includes the ledger state, and `check` carries a
`problems` array and exits non-zero when it is not empty. `--quiet`
suppresses successful human output only. `-f` remains `--file`; format has
no short `-f`.

## Three lists

- **queue** — short-term. File order is priority. `active` is `queue[0]` or `null`.
- **archive** — done or dropped. Newest first. IDs never reused.
- **horizon** — mapped work with no start condition (`research`, `evaluation`,
  `deferred`). `open` names the missing condition. Not priority. Never `active`.

The three lists partition one global task corpus. Every id from `PREFIX-001`
through the highest id exists exactly once across them. `qctl check` reports
missing numbers, duplicate rows, and an id claimed by more than one status.
Moving a row changes its status; it never creates another copy.

Promote horizon → queue with `qctl promote ID -a …` after `open` is
resolved. Do not start a horizon id.

`schema_version` is 4. `notes` is a list. A schema 3 file is rewritten by
`qctl fmt`; other verbs refuse it.

Acceptance is closure truth stated in advance: observable, demonstrably true end
conditions written as the resulting state. For example, `The schema page names
every row field.` is acceptance; `Run the schema generator` is an implementation
step. Put the procedure in `plan` and supporting context in `notes`.

## Mutate

Do not splice a new `- id:` into `tasks.yaml`. `add` creates queue rows;
`add --horizon` creates unfleshed horizon rows; `park ID` demotes queue →
horizon. Prefer every other verb over a YAML edit: `archive` also takes the
archived id out of every `blocked_by` that named it. `edit ID` updates
fields and queue position on an existing row.

```sh
qctl add -t 'Document the schema' -s docs -o 'Readers can verify every row field.' -a 'The schema page names the queue, archive, and horizon fields.'
qctl add -t 'Document the schema' -s docs -o 'Readers can verify every row field.' -a 'The schema page names every row field.' -n 'The type definitions are authoritative.' -b QCTL-001 -A QCTL-001
qctl add -t 'Choose storage' -s design -o 'One storage decision is recorded.' -H -k research -O 'The benchmark result is missing.'
qctl park QCTL-001 -k research -O 'The benchmark result is missing.'
qctl promote QCTL-001 -a 'The decision record names the selected storage.'
qctl edit QCTL-001 -n 'An addendum' -t 'New title'
qctl edit QCTL-001 -x note:1 -p before:QCTL-002
qctl start QCTL-001
qctl archive QCTL-001 -e 'Shipped.'
qctl fmt
qctl close-from-git
qctl hook install
```

`edit -x/--remove` accepts `note`, `acceptance`, `link`, `blocked-by`, or
`evidence` plus a 1-based index or exact text. `edit -p/--position` accepts
`front`, `back`, `before:ID`, or `after:ID`; every dependency must still point
to an earlier queued row.

`--plan` must be a file next to the ledger. `add --horizon` and `park`
write `horizon:` if the ledger omitted it. `close-from-git` archives
queued ids closed by `Closes PREFIX-NNN` / `Completes: PREFIX-NNN` in
the commit **body**. `hook install` prints the `mise run q close-from-git`
snippet to add under Lefthook `pre-push.commands` when `lefthook.yml`
exists; it does not edit that file, and exits non-zero until Lefthook
already runs it. Otherwise it writes a git pre-push. `--force` overwrites
the git hook only. Neither amends.

Optional `patch:` is a forkctl patch name. When `active` has one, select
that patch before editing.

`$schema` is a **pinned** qctl URL. `init` must not copy a schema file into
the consumer.

## Stop conditions

Stop and ask when the work has no start condition (horizon, do not queue),
when `active` would point at horizon, or when installed help disagrees with
remembered syntax.
