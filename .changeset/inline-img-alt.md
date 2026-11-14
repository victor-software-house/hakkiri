---
hakkiri: patch
"verctl:group": fixed
---

The alt text of an `img` tag in a line of Markdown reads as an `alt` block, as Vale reads it.
A rule on `text` or `alt` now checks it, and the paragraph around it reads as before.
