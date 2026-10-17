#!/usr/bin/env bash
#MISE description="semver, allowing only the QCTL-039 API changes while 0.5.8 is the newest release"
# CI's Semver step. QCTL-039 adds two `Style` fields and `Change::Items`, and
# `Document::new` and `Document::append` change arity, in a patch release:
# qctl releases are patch-only. While the newest qctl on crates.io is 0.5.8, a
# failing check passes only when every failed item is one of those, so any other
# break still fails. Once 0.5.9 is published the full check runs; delete this
# task and call `semver` from CI then.
set -euo pipefail
newest=$(curl -fsS -A qctl-ci https://crates.io/api/v1/crates/qctl | jq -r .crate.max_stable_version)
if [ "$newest" != "0.5.8" ]; then
  exec cargo semver-checks
fi
if out=$(cargo semver-checks --color never 2>&1); then
  printf '%s\n' "$out"
  exit 0
fi
printf '%s\n' "$out"
items=$(printf '%s\n' "$out" | grep -E '^  [^ ]' || true)
other=$(printf '%s\n' "$items" | grep -v \
  -e '^  field Style\.list_items in ' \
  -e '^  field Style\.quotes in ' \
  -e '^  variant Change:Items in ' \
  -e '^  qctl::document::Document::new takes 1 parameters in ' \
  -e '^  qctl::document::Document::append takes 3 parameters in ' || true)
if [ -z "$items" ] || [ -n "$other" ]; then
  echo "✘ semver failures beyond the QCTL-039 API changes:" >&2
  printf '%s\n' "${other:-no failed items found}" >&2
  exit 1
fi
echo "✔ only the QCTL-039 API changes fail, which this release allows"
