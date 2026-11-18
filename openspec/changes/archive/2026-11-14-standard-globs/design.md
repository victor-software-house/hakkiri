# Design

## Decisions

### 1. Globs follow gitignore

A glob follows the rules of [gitignore][gitignore], the pattern format that git and ripgrep share.  
globset compiles it with a separator that `*` does not cross.  
When a glob has no `/`, hakkiri puts `**/` before it.  
It then matches the file name in any folder, as gitignore and EditorConfig match it.

hakkiri drops a leading `/`, which ties the glob to the root.  
It adds `**` after a trailing `/`, so the glob matches every file in the folder.  
The specificity of the settings cascade still reads the glob as written.

Alternative: follow Vale.  
Its `*` crosses `/` in a glob with no `**`.  
In a glob that has `**`, its trailing `/**` matches zero folders.  
The operator ruled on 2026-11-14 that a glob follows the standard, whether Vale is looser or stricter.

[gitignore]: https://git-scm.com/docs/gitignore#_pattern_format
