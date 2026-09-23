# Adopt the ctl-core 0.6 presentation defaults

## Why

qctl pins ctl-core 0.5.2. ctl-core 0.6.2 ships the look chosen in its
`choose-visual-identity` change: borderless records, an identifier role, pretty
JSON, a two-column automatic-width buffer, and an 80-column fallback when no
width is detected. On 0.5.2 a piped `qctl show` of a row with a 104-character
title prints one 150-column line, which agent and CI captures then clip.

Queue row: QCTL-037.

## What changes

1. Pin ctl-core `=0.6.2`.
2. Row ids render through the identifier role (`Text::id`, `Table::id_column`).
   Paths stay tokens.
3. Integration tests that match message text set a wide `COLUMNS`, so a message
   is not split by the fallback width. One test proves the fallback instead.

## Impact

Pretty output changes shape: records lose their box and JSON is indented.
The JSON data itself does not change.
