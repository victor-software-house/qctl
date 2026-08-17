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
| Rust | `#c45c2a` | The active task, and the same accent on the banner chip |

The square mark has one rust shape. The banner repeats that rust on the
chip so the header matches the mark. It is the same accent, not a second
role.

Banner-only tints: `#6f675c` (muted on cream), `#8d857a` (muted on ink),
`#ddd6c8` / `#2f2f2f` (hairline).

## Construction

A 32-unit square, corner radius 6, mirrored about `y 16` and inset 4.5 units
left and right, 5 top and bottom. The rule is 3 units wide at `x 14.5`, running
`y 5` to `27`. Two queued cells, 8×8, sit at `x 4.5` on `y 6.5` and `y 17.5` —
their midpoint is the rule's centre, so the pair reads as balanced against it.
The task in flight is 8×10 at `x 19.5`, centred on the same line and taller
than either queued cell. Both side gaps to the rule are 2 units; closing them
collapses the split, which is the whole statement.

The rule is 3 units and not 2 for one reason: at 16px a 2-unit stroke lands on
a single half-covered pixel and renders pale grey, so the split — the entire
point of the mark — is the first thing to disappear. 3 units matches the
connector weight in the `forkctl` mark and holds.

## Banner text

Set in [Geist Mono](https://github.com/vercel/geist-font) (OFL) and converted
to outlines, so nothing depends on a font at render time: wordmark Black 60px
with −3 tracking, tagline Medium 17px, chip Regular 16px. To change the
wording, reshape with `fonttools` + `uharfbuzz` at those sizes rather than
adding a `<text>` element.
