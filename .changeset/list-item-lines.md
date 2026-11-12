---
hakkiri: patch
"verctl:group": changed
---

A fault about one item of a list in the config names the line of that item.
The items are a `styles` name that is no style, a `vocab` name that is no vocabulary, and a pattern of `block_ignores` or `token_ignores` that does not compile.
Before, each fault named the first line of the list.
