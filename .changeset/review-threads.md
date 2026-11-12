---
hakkiri: patch
"verctl:group": fixed
---

`hakkiri test` reads a case through the config with the config's block and token ignores, as `hakkiri check` reads a file.
A level from the config or from a `hakkiri Style.Rule: level` comment leaves a rule failure an error, so a broken rule still fails the run.
A vocabulary term keeps its group names, backreferences and flags, so `(?<Dup>Ab)\k<Dup>` and `(?U)` build as written.
A `$` span beside a stray backtick, a code span or another span reads as math, a line indented four columns opens no display block, and a paragraph of unclosed `$` or `<` costs its length.
A line with no `>` in a quote holds none of the containers inside the quote, so a `$$` line indented six spaces after `> - Item` opens no display block.
A file that `formats` maps has the extension of its reader in a scope such as `text.md`, so that scope selects text in a `.txt` file mapped to `md`.
