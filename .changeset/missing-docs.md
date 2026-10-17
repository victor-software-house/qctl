---
"qctl": patch
---

Every public item is documented and implements `Debug`.
`--help` now describes every flag and argument, including `--title`, `--scope`, `--outcome`, `--patch`, `--plan`, `--kind`, `--open`, and the task id.
The `--disposition` and `--unset` values say what each one means.
The generated schema gives each `section_order` value a description, as it already did for the other enumerations.
