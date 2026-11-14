---
hakkiri: patch
"verctl:group": changed
---

A `files` or `formats` glob matches by the rules of gitignore: `*` stays inside one folder, `**` spans folders, and a glob with no `/` matches the file name in any folder.
So `docs/*.md` no longer matches `docs/x/a.md`: write `docs/**/*.md` for every file below `docs`.
A leading `/` ties a glob to the project root, and a trailing `/` matches every file the folder holds.
