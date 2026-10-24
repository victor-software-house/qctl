# plain-json Specification

## Purpose
Keep machine output plain: a caller that asks for `--format json` and pipes it to `jq` never receives an ESC byte, whatever colour variables its shell sets.

## Requirements

### Requirement: JSON output holds no ANSI under forced colour

With `CLICOLOR_FORCE=1`, `FORCE_COLOR=3`, and `COLORTERM=truecolor`, every invocation that asks for `--format json` SHALL write JSON to stdout and SHALL write no ESC byte to stdout or stderr.

#### Scenario: Success

- **WHEN** `qctl status --format json --file tasks.yaml` runs under forced colour
- **THEN** stdout is JSON and neither stream holds an ESC byte

#### Scenario: Parse error before the format flag

- **WHEN** `qctl status --bogus --format json` runs under forced colour
- **THEN** stdout is a JSON error and neither stream holds an ESC byte
