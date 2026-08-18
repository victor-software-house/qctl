# Changelog

## qctl 0.0.2

- Release through verctl: CI on every PR, a Version PR on main, and
`darwin-arm64` plus `linux-x64` tarballs on the GitHub Release.
- Keep Cargo.lock in step with the bump. The Version PR now regenerates it and
commits it, so `cargo package --locked` no longer fails Verify on a dirty tree.

