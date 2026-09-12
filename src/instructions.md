# qctl agent instructions

`qctl` maintains one in-repo YAML work queue. The file is the ledger.
Session todos, chat recap, and GitHub issues are not.

## Invocation

Prefer the consumer's mise-provisioned task when it exists. Do not invent
aliases for rejected commands.

```
mise run q status
mise run q check
mise run q instructions
```

The `--` in `#USAGE mount` is mise's completion bootstrap. Do not put
it in front of verbs.

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
  id is a check failure. A git scan that cannot run is a check failure, not
  a skip. `--no-git` is for a scratch ledger that is not in the current
  repository. `qctl close-from-git`
  archives those ids (evidence is the commit SHA). `qctl hook install`
  prints the Lefthook `pre-push` command (`mise run q close-from-git`) when
  `lefthook.yml` exists; it does not edit that file, and exits non-zero until
  Lefthook already runs `close-from-git`. Otherwise it writes a
  git `pre-push` that runs
  `qctl close-from-git --pre-push`. Neither amends. `Closes #12` is
  GitHub's, not ours.
- `qctl instructions` and `--help` are the installed-version contract.

## Output

Every command returns one typed report. Human output is pretty by default.
`--color never` and `--no-color` render the same document without ANSI.
`--format json` serializes the report directly as one JSON line on stdout;
failures remain machine-readable on stdout and exit non-zero. `--quiet`
suppresses successful human output only, never JSON or failures. `-f` remains
`--file`; format has no `-f` shorthand. `--color` has no `-c` shorthand;
`fmt -c` is `--check`.

`status --format json` includes the complete ledger state. The JSON from
`check --format json` carries `problems` as an array and exits non-zero when it
is not empty.

## Three lists

| List | Meaning |
|:--|:--|
| `queue` | Short-term work. File order is priority. Exactly one `active`, or `null`. |
| `archive` | Finished or dropped. Newest `completed` first. IDs never reused. `notes` stay. |
| `horizon` | Mapped but not startable: research, evaluations, deferred. File order is not priority. `active` must never name a horizon id. |

Horizon rows require `kind` (`research` / `evaluation` / `deferred`) and
`open` (the missing start condition or the question). Promote to `queue`
only when `open` is resolved and the row has `acceptance` and `blocked_by`.

IDs are `{prefix}-NNN` (at least three digits), never reused, and never encode
priority. Queue, horizon, and archive partition one task corpus: every id from
`{prefix}-001` through the highest id exists exactly once across those lists. A
move changes status without creating a second copy; `qctl check` reports every
gap and every id claimed by more than one status.

`schema_version` is `4`. `notes` is a list of strings on every row. A
schema 3 ledger still has scalar notes: run `qctl fmt` once to split those
on blank-line paragraphs and set version 4. Other verbs refuse version 3
and name that command. A ledger on an earlier version is not rewritten.

Acceptance is closure truth stated in advance. Each line names an observable,
demonstrably true end condition and reads as the resulting state, for example:
`The generated schema names every row field.` Do not put commands to run,
implementation instructions, or a sequence of steps in acceptance. A `plan`
owns the procedure; `notes` carry context a reader cannot reconstruct.

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

`fmt` does not guess. On a current-version file it changes what an option
names, sorts the archive when asked, and removes whitespace nobody chose. It
never adds a blank line, a comment, or a key, and it never rewrites a
value. Schema 3 is the exception: scalar `notes` become a list, then the
style rules run.

Changing `timezone` does not move the stamps already written: nothing records
which zone an old stamp was taken in. Change it deliberately.

## Workflow

1. `qctl status` then `qctl check` before mutation.
2. `qctl add` writes a queue row and prints the new id. Repeatable `-n/--note`,
   `-b/--blocked-by`, `-l/--plan` and `-L/--link` fill the fields a hand edit
   used to. `-A/--after ID` / `-B/--before ID` place it; the tail is the
   default. A blocker that would not sit earlier than the new row is
   refused. `-H/--horizon -k KIND -O OPEN` writes a horizon row
   instead. `--plan` must name a file next to the ledger, the same
   rule `check` uses. A tail add to an empty queue is allowed even
   when `active` is set; it does not invent a `--before`.
3. `qctl start ID` requires the id to be queued and unblocked; it becomes
   `queue[0]` and `active`.
4. `qctl archive ID -e EVIDENCE` moves a queued row to archive.
   `qctl close-from-git` archives every queued id a commit-body trailer
   closed. Default scan is the same history `check` uses. `-r/--range
   main..HEAD` narrows it. `-u/--pre-push` reads the hook stdin. If it
   wrote the ledger, it exits non-zero so the file can be committed;
   it does not amend.
5. `qctl hook install` prefers Lefthook: it prints `mise run q close-from-git`
   for `pre-push.commands` and does not edit `lefthook.yml`. Until that command
   is in Lefthook, install exits non-zero. Without Lefthook it writes
   `.git/hooks/pre-push`. `-F/--force` overwrites the git hook only.
6. `qctl park ID -k KIND -O OPEN` demotes a queued row onto the horizon,
   dropping `acceptance` and `blocked_by`. It refuses if another queued
   row still names that id as a blocker. Parking `active` nulls `active`.
   `qctl promote ID -a 'The decision record names the selected storage.'`
   moves a horizon row onto the queue tail, dropping kind and open. It does
   not become active. `--blocked-by` must name a queued id. `add --horizon`
   creates an unfleshed horizon row; `park` no longer creates. Both write a
   `horizon:` key if the ledger omitted it. `fmt` still never inserts a key.
7. `qctl edit ID` rewrites named fields on a queued, horizon, or archived
   row. Scalars replace (`-t/-s/-o`, `-k/-O`, `-P/-l`, `-U/--unset
   patch|plan`). Lists append (`-a/-n/-L/-b/-e`). Repeatable
   `-x/--remove field:index|exact-text` drops notes, acceptance, links,
   blockers, or evidence; the field names are `note`, `acceptance`, `link`,
   `blocked-by`, and `evidence`. `-R/--reorder-notes 3,1,2` permutes notes.
   `-p/--position front|back|before:ID|after:ID` moves a queued row without
   stealing `active`; every dependency must still point backward. Archive
   rows accept only note, evidence, and link operations. `id` and `completed`
   are never editable. A missing id fails.

YAML: quote list items that start with `#` or contain `: `.

`add` / `start` / `archive` / `park` / `promote` / `edit` change only the
lines they must, so a comment, a blank line, a folded scalar and an inline
list all survive a verb. Do not splice a new `- id:` into the file: `add`
creates rows; `add --horizon` creates unfleshed horizon rows; `park`
demotes. Prefer the verb over a hand edit: `archive` also takes the
archived id out of every `blocked_by` that named it, which a hand edit
forgets. `edit` is the verb for notes, acceptance, and the other fields
on an existing row.

## Stop conditions

Stop and ask when:

- a row has no clear short-term start condition — put it on `horizon`;
- `active` would name a horizon id;
- a blocker is not an earlier queued id;
- installed help differs from remembered syntax.
