---
hakkiri: patch
"verctl:group": changed
---

Each file, fenced code block and comment has a language from langdex, the GitHub Linguist table.
A file takes the language of its path, or of the interpreter on its first line, and is read by the reader of that language, so `.mts`, `.cts`, `.markdown` and a script that starts `#!/usr/bin/env node` get their reader.
The last piece of a scope part names a language by any of its names, so `text.comment.js` reaches `.jsx`, `.mjs` and `.cjs` files, and `text.txt` no longer reaches a `.rst` file or a `README`.
A fenced code block of a language whose comments hakkiri reads is parsed with that grammar, and its comments are linted.
`formats` takes any name of a language hakkiri reads, and `READERS` is gone.
