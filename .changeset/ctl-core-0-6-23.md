---
"qctl": patch
---

qctl builds on ctl-core 0.6.23 and yamled 0.0.7, and reads YAML through serde-saphyr 2.0.0.
A list item that needs quotes and holds an apostrophe is now written in double quotes, `"The operator's call: yes."`, instead of doubling the apostrophe inside single quotes.
