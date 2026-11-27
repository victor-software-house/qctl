---
"qctl": minor
---

qctl builds on ctl-core 0.8.0-rc.2, and `add` and `edit` take a row from a document.
Every command takes `--input FILE`, or `--input -` for standard input, and `--input-format yaml|json`, also read from `QCTL_INPUT` and `QCTL_INPUT_FORMAT`.
A document's keys are long option names and positional ids, and a list value is a list: `qctl add --input row.yaml` with `title`, `scope`, `outcome`, `acceptance` and `note` writes the row the same flags write.
The command line wins over the environment, the environment over the document, and the document over the default; a key that loses is dropped, and a key that names no argument fails with its file and line.

Output changes with ctl-core 0.8, and a caller that reads it must change:

- Piped output defaults to `tsv`, tab-separated lines. `--format pretty` or `QCTL_FORMAT=pretty` gives the table, and `csv` and `markdown` are new formats.
- `--json` is a shorthand for `--format json`, and `--stream` writes each record as the command produces it. Neither has a short form.
- A JSON failure goes to stderr and stdout stays empty, so `qctl check --format json` writes a non-empty `problems` array to stderr.
- A parse error reads `qctl: <message>`, without Clap's `error:` prefix.
- A handler error's JSON `message` is its outermost error, and the rest of its chain is in `causes`.
- A refused input's JSON `message` is `<input> has <n> problems`, and the lines it carried are in `problems`.
- A table never narrows below its column floor, and a table that cannot fit at its floors stacks, each row drawn as a record.
- Help lists `--json`, `--stream`, `--input` and `--input-format`, and `--format` no longer shows `[default: pretty]`.
- A problem in a ledger or a document uses ctl-core 0.8's wording and positions, such as `duplicate key \`prefix\`` for a repeated key, and a field a root mapping lacks sits on line 1, column 1.
