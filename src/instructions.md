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
| `archive` | Finished or dropped. Newest `completed` first. IDs never reused. `notes` stay. |
| `horizon` | Mapped but not startable: research, evaluations, deferred. File order is not priority. `active` must never name a horizon id. |

Horizon rows require `kind` (`research` / `evaluation` / `deferred`) and
`open` (the missing start condition or the question). Promote to `queue`
only when `open` is resolved and the row has `acceptance` and `blocked_by`.

IDs are `{prefix}-NNN` (at least three digits), unique across all three
lists, never reused, never encode priority.

`schema_version` is `3`. An archived row's `completed` is the moment it left
the queue, `YYYY-MM-DDThh:mm:ss`, to the second, in the zone the ledger
declares — so two rows closed in the same session are told apart and
ordered. A ledger on an earlier version has to be migrated before this qctl
will touch it.

## Style

A ledger declares how it is written, under `style`. Every option is optional
and defaults to what qctl already wrote, so a file with no `style` block is
valid.

| Option | Default | What it decides |
|:--|:--|:--|
| `timezone` | `+00:00` | The offset `completed` is written in. Stamps carry no offset of their own. |
| `section_order` | `[queue, archive, horizon]` | The order the three lists appear in. |
| `indent` | `2` | How far a row sits under its list's key. |
| `archive_order` | `newest_first` | Whether `fmt` sorts the archive or leaves it as written. |
| `normalize_on_write` | `false` | Whether every verb normalizes the whole file, or only the lines it changes. |

`qctl fmt` writes a ledger in its declared style. `qctl fmt --check` writes
nothing, names the lines that differ, and exits non-zero — that is the one
for a hook.

`fmt` does not guess. It changes what an option names, sorts the archive when
asked, and removes whitespace nobody chose. It never adds a blank line, a
comment, or a key, and it never rewrites a value.

Changing `timezone` does not move the stamps already written: nothing records
which zone an old stamp was taken in. Change it deliberately.

## Workflow

1. `qctl status` then `qctl check` before mutation.
2. `qctl add` writes a queue row and prints the new id. `--notes`,
   `--blocked-by`, `--plan` and `--link` fill the fields a hand edit
   used to. `--after ID` / `--before ID` place it; the tail is the
   default. A blocker that would not sit earlier than the new row is
   refused. `--horizon --kind KIND --open OPEN` writes a horizon row
   instead.
3. `qctl start ID` requires the id to be queued and unblocked; it becomes
   `queue[0]` and `active`.
4. `qctl archive ID -e EVIDENCE` moves a queued row to archive.
5. `qctl park` writes a horizon row (`--kind` and `--open` required).
   `qctl promote ID -a ACCEPTANCE` moves it onto the queue tail, dropping
   kind and open. It does not become active. `--blocked-by` must name a
   queued id.

YAML: quote list items that start with `#` or contain `: `.

`add` / `start` / `archive` / `park` / `promote` change only the lines they
must, so a comment, a blank line, a folded scalar and an inline list all
survive a verb. Prefer the verb over a hand edit: `archive` also takes the
archived id out of every `blocked_by` that named it, which a hand edit
forgets.

## Stop conditions

Stop and ask when:

- a row has no clear short-term start condition — put it on `horizon`;
- `active` would name a horizon id;
- a blocker is not an earlier queued id;
- installed help differs from remembered syntax.
