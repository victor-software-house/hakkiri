---
hakkiri: patch
"verctl:group": fixed
---

A plain-text file whose first 8192 bytes are not valid UTF-8, such as a Latin-1 file, is binary and skipped instead of failing the run.
