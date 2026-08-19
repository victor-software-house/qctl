---
qctl: patch
---

Archiving a row from between two others no longer takes their separator with it.

A row's span reaches to where the next row begins, so it already held the blank
line between them; taking the blank line above as well removed two separators for
one row, and the rows either side ended up written against each other. The last
row of a list has no next row, so there the blank above is still the one to take.

Found by reading this repo's own ledger: two rows had lost their separator, and no
fixture archived from the middle of a list. One does now.
