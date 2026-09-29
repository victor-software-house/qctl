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

4. **Reorder around a loose comment instead of refusing.** yamled's `reorder`
   keeps each loose comment and blank line in its slot while the lists trade
   places, so the outcome is defined. The old refusal existed because the
   hand-written splice could not place such a comment. Keeping the refusal
   lost because it made `fmt` fail on a file it can now format without losing
   a byte.
5. **Note headers come from yamled.** A literal note is `|-` and a folded one
   `>-`, with an indentation digit only when the first line starts with a
   space. qctl still decides which of plain, folded, or literal a note gets.
   Keeping `|2-` and `>2-` on every note lost because it needs a rendering
   path of qctl's own beside yamled's.
6. **Rows are spaced by a blank line; a row's own lists are not.** A list
   whose rows already agree keeps its spacing. yamled falls back to one
   document-wide spacing, so qctl switches to tight spacing for the pushes into
   a row's lists.

## Overrides

No automatic behaviour is added. A ledger that used a loose boolean or a
duplicate key was already invalid under the published schema; it now fails at
read instead of at `check`.
