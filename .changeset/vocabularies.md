---
hakkiri: patch
"verctl:group": added
---

A project names vocabularies in `vocab`, at the top level and in a glob of `files`, and hakkiri reads each from `config/vocabularies/<name>/` in a package.
The accepted terms hold back matches in `capitalization`, `conditional`, `existence`, `repetition`, `substitution` and `spelling` rules, as in Vale, and a glob's list does so for the files it matches.
