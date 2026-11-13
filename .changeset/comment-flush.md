---
hakkiri: patch
"verctl:group": fixed
---

A comment that turns rules off no longer reaches text that Vale reads before the comment.
Vale reads the text of a tight list item before a nested list, quote, code block or table.
It reads the text of an HTML block before a tag that opens a block, and the text of a paragraph before a `div` or `p` tag inside it.
A comment after that text does not hide its alerts.
A comment in a block of its own, after the last text of an HTML block, reaches that text when no tag flushes it first.
A block tag inside an inline tag, such as a `div` inside a `span`, is read as inline, so it flushes no text.
