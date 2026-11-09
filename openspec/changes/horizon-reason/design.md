# Design

## Context

See `proposal.md` for what the field is and where it reaches.
A horizon row is `HorizonTask` in `src/schema.rs`, and `deny_unknown_fields` makes every row type refuse a key it does not know, so a renamed key is a new schema version.
`qctl fmt` already upgrades older ledgers in `src/format.rs`: schema 3 rewrites scalar notes, schema 4 sets the version, and the other verbs refuse both with a message that names `qctl fmt` (`stale_schema_message` in `src/ledger.rs`).

## Goals / Non-Goals

**Goals:**

1. The stored key, the flag, its variable, the JSON field, and every message say `reason`.
2. One `qctl fmt` run moves any ledger qctl can upgrade to schema 6, and changes nothing but the renamed keys and the version line on a ledger already in its declared style.

**Non-Goals:**

1. New values. `research`, `evaluation`, and `deferred` stay.
2. Rewriting the archived history in a ledger's notes and evidence, which may still say "kind".
3. Upgrading consumer ledgers. Each consumer runs `qctl fmt` in the pull request that takes the release.

## Decisions

### 1. Rename the key where it is written

`fmt` finds each horizon row's `kind` key through yamled and rewrites only the key's bytes, through `rename_row_keys` in `src/document.rs`.
The value, its quotes, a trailing comment, and the comments above it keep their bytes.

Alternative: remove `kind` and insert `reason` at the end of the row with yamled's existing edits.
It lost because it moves the field below `open`, drops a trailing comment, and turns a one-word change into a moved line in every consumer's diff.

### 2. No alias and no fallback reader

`HorizonTask` knows only `reason`.
The upgrade in `fmt` is the one place that reads `kind`, and every other verb refuses an older ledger and names `qctl fmt`.

Alternative: a serde alias that accepts `kind` on read.
It lost because the ledger would keep the old key forever, the generated schema would still refuse it in editors and `check`, and two spellings of one field is drift waiting to happen.

### 3. Schema 3 and 4 take the same rename

A schema 3 or 4 ledger also says `kind`, so its upgrade ends with the same rename and the version line.
A ledger upgrades in one run whatever older version it declares.

### 4. The short flag is -r

`-r` is unused on `add`, `park`, and `edit`, and the global options take no short `-r`.
`close-from-git -r/--range` is a different verb, so the letters do not meet.

Alternative: `-R`.
It lost because `edit` already takes `-R/--reorder-notes`.

## Risks / Trade-offs

1. A caller that passes `-k` or sets `QCTL_KIND` fails or is ignored after the release. The changeset names both spellings so the consumer pull requests change them.
2. The pinned `mise run q` task in this repository runs the released qctl, which refuses schema 6 until the Version PR moves that pin.
