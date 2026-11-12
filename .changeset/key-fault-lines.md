---
hakkiri: patch
"verctl:group": changed
---

A fault about a key itself in the config names the line of that key, also when the value below it spans lines.
The keys are a glob that is no glob, a setting that names no rule, a key that a package's settings file may not hold, and a `path` entry in such a file.
Before, the fault named the first line of the value.
