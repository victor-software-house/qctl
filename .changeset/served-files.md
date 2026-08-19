---
qctl: patch
---

Every version a consumer reads out of a tag now comes from the release that tag
names. `tasks/q/q` and `examples/mise.toml` are rendered from
`.verctl/templates/`, and README's two spellings — the `?ref=` include and the
`mise x …@` install line — are declared as `[patterns]`, so the Version PR
rewrites all five sites on the commit the tag points at.

Three tags shipped a task file that installed the previous release: `git show
v0.1.0:tasks/q/q` pinned `0.0.1`, because every site was a hand bump somebody
had to remember. A rendered file cannot drift, and a declared pattern that stops
matching stops the release instead of shipping quietly.

`examples/mise.toml` also gained the `[tools]` entry it was missing, so copying
it installs the pinned binary rather than whatever the task include happens to
carry.
