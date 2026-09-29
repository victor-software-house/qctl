# Design

## Decisions

1. **Keep JSON Schema in `check`, place its errors through yamled.** The
   alternative was to validate `check` with garde through `Input::check`, as
   the verbs do. It lost because the published `tasks.schema.json` is the
   contract other tools read, and `check` proves a file against that contract.
   yamled's `locate_nearest` turns each instance path into a line.
2. **Allow `ctl_core::input` in domain modules.** The architecture test keeps
   domain modules away from ctl-core because ctl-core owns presentation.
   Declared input is not presentation. The alternative, routing every parse
   through `src/ledger.rs` wrappers, lost because it re-exports the same type
   under another name. The test now forbids every other `ctl_core::` path.
3. **Two pull requests, reads first.** Reads change error text only; edits
   change written bytes. Splitting them keeps each snapshot diff attributable
   to one cause.

## Overrides

No automatic behaviour is added. A ledger that used a loose boolean or a
duplicate key was already invalid under the published schema; it now fails at
read instead of at `check`.
