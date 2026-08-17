# Mark

The mark is the invariant: a rule down the middle, the queue on one side, and
exactly one task in flight on the other — larger, and the only rust in the
file. Same palette and 32-unit grid as `verctl` and `forkctl`; each tool's
topology differs — a rising history there, a fork graph in `forkctl`, this
split here.

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

A 32-unit square, corner radius 6, mirrored about `y 16`. The rule is 2 units
wide at `x 15`, running `y 4` to `28`. Two queued cells, 9×8.5, sit at `x 3.5`
on `y 6` and `y 17.5` — their midpoint is the rule's centre, so the pair reads
as balanced against it. The task in flight is 9×11 at `x 19.5`, centred on the
same line and taller than either queued cell. Both side gaps to the rule are
2.5 units; closing them collapses the split, which is the whole statement.

## Banner text

Set in [Geist Mono](https://github.com/vercel/geist-font) (OFL) and converted
to outlines, so nothing depends on a font at render time: wordmark Black 60px
with −3 tracking, tagline Medium 17px, chip Regular 16px. To change the
wording, reshape with `fonttools` + `uharfbuzz` at those sizes rather than
adding a `<text>` element.
