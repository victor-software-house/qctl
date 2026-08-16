# Mark

The mark is the queue: one file as a rail, rows attached in order, the active
task full width and in rust at the head. The queued rows shorten as they
recede. Same palette and 32-unit grid as `verctl` and `forkctl`; each tool's
topology differs — a rising history there, a carried stack in `forkctl`, an
ordered queue here.

| File | Use |
|:--|:--|
| [`mark.svg`](mark.svg) | Square mark, cream field. Avatar, favicon, docs. |
| [`mark-dark.svg`](mark-dark.svg) | Same geometry, ink field. |
| [`banner.svg`](banner.svg) | README header, 1200×240. |
| [`banner-dark.svg`](banner-dark.svg) | Same, ink field. |

Pair the two fields with `<picture>` and `prefers-color-scheme`, as the README
header does. Never recolour a single file at the call site.

## Palette

| | Hex | Role |
|:--|:--|:--|
| Cream | `#f3efe6` | Field, or figure on ink |
| Ink | `#161616` | Figure, or field |
| Rust | `#c45c2a` | The active task. One accent, never two |

Banner-only tints: `#6f675c` (muted on cream), `#8d857a` (muted on ink),
`#ddd6c8` / `#2f2f2f` (hairline).

## Construction

A 32-unit square, corner radius 6. The file is a 3-unit rail at `x 4`, full
height. Three rows start at `x 9`: the active one 19 units wide and 8 tall at
`y 4`, then 15 units at `y 14.5` and 11 units at `y 22.5`. The 2.5 and 2-unit
row gaps are load bearing: closed up, the rows render as one block at 16px,
which is the size it gets judged at.

## Banner text

Set in [Geist Mono](https://github.com/vercel/geist-font) (OFL) and converted
to outlines, so nothing depends on a font at render time: wordmark Black 60px
with −3 tracking, tagline Medium 17px, chip Regular 16px. To change the
wording, reshape with `fonttools` + `uharfbuzz` at those sizes rather than
adding a `<text>` element.
