---
hakkiri: patch
"verctl:group": added
---

A glob of `files` takes `block_ignores` and `token_ignores`, each a list of patterns, and a package's settings file can hold them too.
In a Markdown or MDX file, each match of a block pattern is read as a fenced code block that holds its first group.
Each match of a token pattern is read as a code span that holds it.
No rule that reads prose reports text inside them.
The patterns of every glob that matches a file apply, a package's first.
