---
qctl: minor
---

A ledger can declare `style: {row_spacing: blank_line}`, and `qctl fmt` then keeps exactly one blank line between rows: a lost separator is put back, and `fmt --check` reports it. The first row stays tight to its list's key, the blank line goes above a row's comments, and a row ending in a `|+` or `>+` block is left alone. The default, `as_written`, changes nothing.
