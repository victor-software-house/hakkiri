---
hakkiri: patch
"verctl:group": fixed
---

Add `code_span_size`, the number of words a code span counts as in a word count.
It is 1 by default, and 0 counts as Vale does.
It reaches the `words`, `sentences` and `syllables` of a `metric` or `readability` rule, and the scores built on them.
A package's settings file, the project config and a glob set it, and `--code-span-size` and `HAKKIRI_CODE_SPAN_SIZE` win over them.
`engine::lint_accepting` and `summary::summarize` take it as a last argument.
