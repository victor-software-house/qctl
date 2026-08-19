# qctl

Rust policy CLI for in-repo `tasks.yaml` work queues.

- Operator contract: `qctl instructions` and `skills/qctl/SKILL.md`.
- This repo's queue is [`tasks.yaml`](tasks.yaml) (`QCTL-###`).
- `horizon` maps research/evaluations that are not startable. Do not put
  them on `queue` and do not set `active` to a horizon id.
- Schema is types + schemars (QCTL-001). Generated JSON lives only here.
  Consumers pin a `$schema` URL and run `qctl check`. `schema_version` is 2:
  `completed` is an instant in UTC, not a day. A rule that both `check` and the
  verbs must agree on is stated once, as a `#[garde(...)]` attribute — schemars
  reads those, so a second `#[schemars(...)]` copy is drift waiting to happen.
- Mutations rewrite only the lines they change (QCTL-002). Each scenario in
  `tests/fixtures/mutations/` records what the verb wrote as one marked-up
  file: `-` removed, `+` added. When a change to that output is intended,
  `mise run snapshots` shows it scenario by scenario and accepts it.
