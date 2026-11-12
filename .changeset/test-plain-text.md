---
hakkiri: patch
"verctl:group": added
---

`hakkiri test` reads a case whose `format` no extension names, such as `txt`, as plain text, where it refused the case.
A case that runs through the config reads its input by the config's `formats` entry first, as `hakkiri check` does for a file.
