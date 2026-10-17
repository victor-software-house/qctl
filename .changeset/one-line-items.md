---
"qctl": patch
---

A ledger's `style` declares `list_items` (`line` or `auto`) and `quotes` (`auto`, `single`, or `double`).
The defaults are `line` and `auto`: every list item a verb writes sits on one line, plain when it reads back unquoted, and `add` and `edit` refuse a `-n/--note` that holds a line break.
`qctl fmt` writes older folded and literal list items on one line, splits a multi-line note into one item per line with its line breaks and blank lines dropped, and re-quotes every one-line string outside `style` to the declared quotes; every other value reads back the same.
A ledger written before this release changes shape on its first `qctl fmt`; `list_items: auto` keeps block items and multi-line notes.
