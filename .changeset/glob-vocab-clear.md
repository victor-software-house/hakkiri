---
hakkiri: patch
"verctl:group": added
---

A glob's empty `vocab` list drops only the lists that earlier configs gave the same glob text, as Vale's `NONE` clears only the section of that name.
The other globs that match a file still add their lists.
