---
hakkiri: patch
"verctl:group": changed
---

The JSON alert gains `end_line`, and `end_line` and `end_column` name the last character of the match, also for a match that wraps a line.
A wrapped match used to end at the end of its first line.
The pretty line keeps `path:line:column`.
