---
hakkiri: patch
"verctl:group": fixed
---

A file that is not valid UTF-8, such as a Latin-1 one, is linted with each bad byte read as one `?`, so a column still counts bytes, as in Vale.
