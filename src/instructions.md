# qctl agent instructions

`qctl` maintains one in-repo YAML work queue. The file is the ledger.
Session todos, chat recap, and GitHub issues are not.

## Invocation

Prefer the consumer's mise-provisioned task when it exists. Do not invent
aliases for rejected commands.

```
mise run q -- status
mise run q -- check
mise run q -- instructions
```

Until a tagged GitHub Release exists, `mise github:` cannot install a
binary. Use a local or `cargo install --git` build, then `qctl`.

## Sources of truth

- `tasks.yaml` is the durable queue, archive, and horizon.
- The schema is authored as Rust types (schemars). The generated JSON Schema
  is a qctl publish artifact. Consumers set `$schema` to a **pinned** URL
  and do not vendor `tasks.schema.json`.
- `qctl check` validates from the schema embedded in the binary, plus graph
  rules JSON Schema cannot express, plus git trailers. A body line
  `Closes PREFIX-NNN` or `Completes: PREFIX-NNN` that names a still-queued
  id is a check failure. `--no-git` skips the scan. Auto-archive is later.
- `qctl instructions` and `--help` are the installed-version contract.

## Three lists

| List | Meaning |
|:--|:--|
| `queue` | Short-term work. File order is priority. Exactly one `active`, or `null`. |
| `archive` | Finished or dropped. Newest `completed` date first. IDs never reused. `notes` stay. |
| `horizon` | Mapped but not startable: research, evaluations, deferred. File order is not priority. `active` must never name a horizon id. |

Horizon rows require `kind` (`research` / `evaluation` / `deferred`) and
`open` (the missing start condition or the question). Promote to `queue`
only when `open` is resolved and the row has `acceptance` and `blocked_by`.

IDs are `{prefix}-NNN` (at least three digits), unique across all three
lists, never reused, never encode priority.

## Workflow

1. `qctl status` then `qctl check` before mutation.
2. `qctl add` appends an unblocked queue row and prints the new id.
3. `qctl start ID` requires the id to be queued and unblocked; it becomes
   `queue[0]` and `active`.
4. `qctl archive ID -e EVIDENCE` moves a queued row to archive.
5. Horizon is edited in the yaml until `qctl` grows promote/park verbs.

YAML: quote list items that start with `#` or contain `: `.

`add` / `start` / `archive` currently rewrite via serde and can drop
comments. Treat that as a known gap (QCTL-002), not as license to hand-edit
around `check`.

## Stop conditions

Stop and ask when:

- a row has no clear short-term start condition — put it on `horizon`;
- `active` would name a horizon id;
- a blocker is not an earlier queued id;
- installed help differs from remembered syntax.
