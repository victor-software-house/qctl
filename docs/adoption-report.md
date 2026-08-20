# What a consumer asked for

A monorepo of twelve independently-versioned packages has run qctl since v0.1.0
and moved to schema 3 without incident. Their ledger is 2,262 lines: 64 queued
rows, one horizon row, a long archive, a 50-line comment block above `queue:`
explaining the ordering rationale, folded scalars throughout, inline lists.

They wrote up what they hit. This is that report, kept because the rows it
produced cite it: **QCTL-023** through **QCTL-029**. The reporter is not named
and their internal details are not reproduced.

## What already works, so it does not get refactored away

**`fmt` preserving hand-written structure is the feature that made qctl usable
for them.** `add`, `start` and `archive` touch only the lines they must, and
`fmt --check` is clean on a file no tool wrote. Their words: that is not a small
thing, and it is why nobody has been tempted to hand-edit around the verbs.

The remote task include works as advertised, including for agents that have
never seen the repo.

## 1. `check` should say which rules it enforces

The most valuable thing qctl could ship for adoption, and it is a document
rather than code.

They still ran a policy engine and a script that restated rules `check` already
enforces. Before deleting them they proved it rather than assuming it: ten
minimal ledgers, one defect each, through `qctl check --no-git`.

| defect | qctl output |
|:--|:--|
| duplicate id | `duplicate id X-001` |
| blocker not queued | `X-002 <- X-777 is not queued` |
| blocker not earlier | `X-002 <- X-003 is not earlier` |
| active not `queue[0]` | `active X-002 must be queue[0] (found X-001)` |
| active is blocked | `active X-001 is blocked` |
| active with an empty queue | `active X-001 is not in the queue` |
| archive out of order | `archive is not newest-first by completed` |
| queue `plan:` with no file | `missing plan docs/nope.md` |
| archive `plan:` with no file | `missing plan docs/archive-nope.md` |
| horizon `plan:` with no file | `missing plan docs/also-nope.md` |

That covered every rule they had, and qctl was **strictly wider**: their script
read `queue` and `archive` only, so a horizon row's `plan:` went unchecked.
qctl chains all three lists, and additionally rejects an id that does not match
the declared prefix and a `completed` stamp that is not a moment that exists.

Their argument: deleting a working check because a replacement *looks*
equivalent is how a gate goes quiet. They only felt safe because they spent an
hour building defect fixtures. Every consumer migrating off a hand-rolled
validator faces that hour, and most will skip it and either keep both forever or
delete on faith.

What removes it: a rule inventory published with the release — every check, its
exact message, and which lists it reads, machine-readable so a consumer can diff
it against their own set. A `check --explain` that prints it and exits 0 does the
job and stays honest, because it lives next to the code.

Filed as **QCTL-023**.

## 2. Horizon has no verbs

`instructions.md` is upfront about this, and they hit it on day one. Writing the
row meant hand-editing YAML, which is what the same document tells agents not to
do.

Worth preserving from their write-up: they call the horizon schema shape **good**
and ask that it not change. Having no `acceptance` and no `blocked_by`
mechanically forced them to split one vague request into a queue row with
provable acceptance and a horizon row with a named blocker — a split they say
they would not have made as cleanly on judgement alone.

`promote` matters more than `park`. Moving a row from horizon to queue means
dropping two fields, adding two, and placing it, with the `active` invariant to
respect.

Filed as **QCTL-024**.

## 3. `add` cannot write a whole row, or place it

`add` takes title, scope, outcome, acceptance and patch. A queued task also has
`notes`, `blocked_by`, `plan` and `links`.

Every row they added needed a hand edit immediately after, always for `notes`,
which is where their conventions put the evidence. So the verb bought them the
id and the placement, and then they edited the file anyway.

Separately: `add` always appends. Queue order **is** priority, so anything urgent
means a hand edit or an immediate `start`, which forces it to the head and makes
it active — usually wrong.

The two halves interact, which is the part worth writing down. Their reasoning
that a blocker is safe to accept unvalidated — a blocker must be earlier, and
`add` appends, so any queued id qualifies — holds only while the tail is the only
landing place. Add placement and `add --before X --blocked-by X` writes a row
ahead of its own blocker, which `check` then rejects: the verb writing a ledger
its own gate refuses. A blocker has to be validated against the resolved
insertion index, not against mere membership.

Filed as **QCTL-025**.

## 4. A check that cannot run must not pass

`trailer_errors` returns no errors when the trailer scan fails, which silently
drops a whole check class in any environment where the git call fails.

The operator-facing half is the same shape: validating a ledger outside the
current repository — a fixture, a scratch copy, a migration staging file — runs
the trailer scan against whatever repo surrounds it. `--no-git` fixes it, and
nothing points there until the odd complaints start.

Filed as **QCTL-026**.

## 5. Every problem prints twice

`check` prints each problem on its own line, then bails with all of them joined.
Ten problems is twenty lines for ten facts, and the joined form is the one that
wraps badly in a terminal.

Filed as **QCTL-027**.

## 6. There is no populated example to copy

Writing their first horizon row by hand, their first attempt used a string where
`evidence` takes an array. The error was good and specific —
`/archive/0/evidence: "Shipped." is not of type "array"` — but they found it by
trial. `init` writes an empty ledger, and there is no fully populated row of each
kind anywhere to copy from.

They name verctl's live, test-parsed `examples/ver.yaml` as the pattern, and call
it the best thing in that project's documentation.

Filed as **QCTL-028**.

## 7. JSON output

`status` and `check` are read by agents far more often than by humans, and
parsing a table is fine until the format shifts.

Filed as **QCTL-029**.

## One ask that was declined

They suggested `check` should reject a horizon row whose `open` condition has
clearly been resolved, and hedged it themselves as "not obviously qctl's
business". Declined: it asks the tool to judge whether prose is still true.
