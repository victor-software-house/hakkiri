---
hakkiri: patch
"verctl:group": changed
---

Front matter is read through the `front-matter` crate, with one parser per format: YAML, TOML and JSON.
hakkiri reads each form that the crate finds, as Vale does where Vale reads it.
YAML sits between `---` lines, after `---yaml` or closed by `...`.
TOML sits between `+++` lines or after `---toml`.
JSON sits between `;;;` lines, after `---json`, or as an object that opens with `{` on the first line.
A YAML key that is a list, a mapping or an alias gives no value and drops none of the entries after it.

Every block is cut from the body, so no line of it is a paragraph.
Vale lints the lines of a `;;;`, `---toml` or bare `{` block and of YAML closed by `...` as a paragraph too, so a key such as `Install` alerts there.
hakkiri lints only the values of the block.
