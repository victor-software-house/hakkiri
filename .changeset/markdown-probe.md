---
hakkiri: patch
"verctl:group": added
---

A Markdown file is read as Vale's converter reads it in more places.
A table cell past the header's last cell, a footnote reference and a definition that nothing references are no text.
A definition that a reference names is a `list` node.
A backslash escape and a hard break written with a backslash lose the backslash, so a rule sees `well-known` and not `well\-known`.
An `ftp://` address is no text, and a bare email address or `www.` address is text.
The `link` scope holds no word for an autolink, a bare URL or a link whose text is its destination.
An inline `b`, `strong`, `i`, `em`, `code`, `tt` or `a` element has the scope of its tag.
