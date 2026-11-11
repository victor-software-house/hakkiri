---
hakkiri: patch
"verctl:group": fixed
---

`hakkiri test` reads a case through the config with the config's block and token ignores, as `hakkiri check` reads a file.
A level from the config or from a `hakkiri Style.Rule: level` comment leaves a rule failure an error, so a broken rule still fails the run.
A vocabulary term keeps its group names, backreferences and flags, so `(?<Dup>Ab)\k<Dup>` and `(?U)` build as written.
A `$` span beside a stray backtick, a code span or another span reads as math, a line indented four columns opens no display block, and a paragraph of unclosed `$` or `<` costs its length.
