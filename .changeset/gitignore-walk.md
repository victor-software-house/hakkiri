---
hakkiri: patch
"verctl:group": added
---

A walk skips the files that `.gitignore`, `.git/info/exclude` and the global git excludes ignore, inside a git repository.
`gitignore: false`, `--no-gitignore`, `--gitignore=false` and `HAKKIRI_GITIGNORE=false` read them.
A file named on the command line is never skipped for that, and one that no glob selects draws a warning: `on_unmatched`, `--on-unmatched` and `HAKKIRI_ON_UNMATCHED` take `warn`, `error` or `ignore`.
`fail_level`, `gitignore` and `on_unmatched` are config keys under the flag and the variable, and a package's settings file that holds one is an error.
