# Design

## Decisions

### 1. Globs follow gitignore

A glob reads by the rules of [gitignore][gitignore], the pattern format git and ripgrep share, and globset compiles it with a separator that `*` does not cross.
A glob with no `/` gets `**/` before it, so it matches the file name in any folder, as gitignore and EditorConfig match it.
A leading `/` is dropped and ties the glob to the root, and a trailing `/` gets `**`, so it matches every file the folder holds.
The specificity of the settings cascade still reads the glob as written.

Alternative: follow Vale, whose `*` crosses `/` in a glob with no `**`, and whose trailing `/**` matches zero folders in a glob that has one.
The operator ruled on 2026-11-14 that a glob follows the standard, whether Vale is looser or stricter.

[gitignore]: https://git-scm.com/docs/gitignore#_pattern_format
