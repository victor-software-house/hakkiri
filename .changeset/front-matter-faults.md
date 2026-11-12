---
hakkiri: patch
---

A Markdown or MDX file whose front matter no parser reads now fails with the line of the problem and exit 2, and the rest of the file is still linted.
A key written twice, a merge key, a key that is a number or a null, and the values `.inf`, `-.inf` and `.NaN` no longer hide the other values of the block, as in Vale.
ctl-core moves to 0.7.4.
