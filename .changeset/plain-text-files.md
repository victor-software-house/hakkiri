---
hakkiri: patch
"verctl:group": added
---

A file that a glob selects and that no reader reads is now linted as plain text, where `hakkiri check notes.txt` was a failure and a walk skipped it.
`formats` maps an extension, a file name or a glob to a reader (`md`, `mdx`, `rs`, `ts`, `tsx`, `js` or `text`), as Vale's `[formats]` does, and a package's settings file may hold it.
`on_unknown`, `--on-unknown` and `HAKKIRI_ON_UNKNOWN` take `text`, `warn`, `error` or `ignore` for a selected file that no reader reads.
A binary file is skipped with no message.
