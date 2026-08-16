# qctl

Rust policy CLI for in-repo `tasks.yaml` work queues. `check` is the
validation command. Do not copy Ajv tests into consumers.

- Prefix lives in the ledger (`prefix: OMX`). IDs are `{prefix}-NNN` with at
  least three digits and are never reused.
- File order is priority. Exactly one `active`, or `null`.
- Mutations should become surgical YAML edits; serde rewrites are a known
  first-slice gap for `start`/`archive`.
- The remote mise catalog is `tasks/q` and execs the released binary.
