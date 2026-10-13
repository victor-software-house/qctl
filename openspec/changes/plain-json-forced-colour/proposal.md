# Keep JSON output plain under forced colour

## Why

Agent shells set `CLICOLOR_FORCE=1`, `FORCE_COLOR=3`, and `COLORTERM=truecolor`.
qctl's JSON output was already plain, but `qctl status --bogus --format json` printed its parse error as coloured text on stderr: ctl-core's lenient pre-parse stopped at the bad argument and never saw the format flag.
A caller piping that to `jq` gets escape bytes instead of JSON.

Queue row: QCTL-045.

## What changes

1. qctl moves from ctl-core 0.6.9 to 0.6.16, which reads a long `--format json` after the bad argument and reports the parse error as JSON.
2. A test runs the binary with forced colour and asserts that no ESC byte reaches stdout or stderr, and that stdout is JSON.

## Impact

1. `Cargo.toml`, `Cargo.lock`, `tests/cli.rs`, operator docs.
