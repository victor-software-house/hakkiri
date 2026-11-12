# Design

## Context

A code span reads as the one-letter word `x` in the text a rule reads ([rule-engine design][engine-text]).
Vale masks it as a run of `*`.
An `occurrence` rule counts no match that overlaps a code span, so the `x` is never counted, as the `match-checks` spec holds.

## Decisions

### 1. The engine keeps the counts of the specs

The probe of Vale at `66bbb85` and of hakkiri (2026-11-11) read `Run `a`, `b`, `c` and `one two three` today.` and `Two words `four word span` more.` through three rules over the sentence scope:

- A `metric` rule with the formula `words`: Vale 3.00 and 3.00, hakkiri 7.00 and 4.00.
- An `occurrence` rule with the token `(?:\b\w+\b|\*+)`: Vale 7 and 4, hakkiri 3 and 3.
- An `occurrence` rule with the token `\b\w+\b`: Vale 3 and 3, hakkiri 3 and 3.

The first number of each pair is for the first text, and the second for the second text.

A `metric` rule counts each code span once in hakkiri and none in Vale, as the spec holds.
An `occurrence` rule counts nothing for a code span in hakkiri, and counts the mask in Vale only when its token reads a `*` run.
Alternative: count a match that overlaps a code span in an `occurrence` rule.
It lost because the `match-checks` spec and its scenario `A match inside code` decide the opposite, and a token such as `x` would count every code span.
Alternative: read a code span as a run of `*` in every rule.
It lost because it brings back the repeats and forms seen in code that [pattern 9][pattern-9] removed.

### 2. The rule moves

`LongSentence` in the prose-style package counts words with a `metric` rule and its `words` formula, which counts each code span once.
Its message shows the count with two decimals, and its cases keep `counts each code span as a word`.

### 3. The key is `code_span_size`, a number of words

The operator chose the name on 2026-11-12.
Its value is the number of words a code span counts as: 1 by default, and 0 to match Vale.
A whole number from 0 to 255 is valid, so a project that counts a span as two words can say so.
Prior art for the shape, read on 2026-11-12:

- Vale lists `IgnoredScopes` (`code`, `tt`), which masks inline code, so it counts no word.
- markdownlint names what a rule includes with booleans such as `code_blocks`.
- remark-retext marks `inlineCode` as a source node, so it is never a word.

Alternative: a boolean such as `count_code_spans`.
It lost because the operator asked for a count, and a boolean cannot say two.
Alternative: a list of scopes to ignore, as Vale does.
It lost because it would reach every rule, where this setting reaches word counts only.

### 4. Word counts only

The setting changes the text that a `metric` or `readability` rule counts words in, and so every variable and score built on that text: `words`, `sentences`, `syllables`, `complex_words`, and the scores.
A rule that reads a match, an `existence` or an `occurrence` rule for one, still reads a code span as one `x`, and a match that overlaps it is still none.
At 0, each code span reads as one `*` in that text, which is no word, as Vale's mask of asterisks is none.
At 2 or more, it reads as that many `x` separated by spaces.
The metric counts `occurrences` of its `tokens` in the text as before.

### 5. Package, project, glob, variable, flag

The last setting wins, in this order of rising weight: the package's settings file, the project config, each matching glob in the order written, `HAKKIRI_CODE_SPAN_SIZE`, then `--code-span-size`.
A glob is the more specific of the two config levels, so it beats the project's top level.
The variable and the flag beat every file, as for the other options of `check`.
`code_span_size` is a reserved key under a glob, as `styles` and `vocab` are, and no rule name can equal it, since a rule name holds a dot.
`hakkiri test` reads the project's key for a case that runs through the config, and runs a case of one rule file at 1.

### 6. Vale drops an alert that has the place of another

Vale keeps each alert under the key of its line, its column and its rule, and drops a later alert with the same key ([history key][v-history]).
A block that is not a sentence is reported at the first column of its line, so a table row of three cells gives one alert, the one of its first cell.
A sentence that starts with a code span has no place in its block, so Vale reports it at the line the block was found on, and an earlier sentence often holds that place.
Vale also finds the line of a block by searching the file for its text ([walker.advance][v-advance]), so a block whose text is not in the file as written can land on the line of another block, and the later alert is dropped.
A reference definition that repeats the text of a code span, and an entity such as `&amp;lt;` in a code span, are two ways to get there.

The probe at 0 differed from Vale in 1,910 alerts.
A build of Vale with that check off, in a box, differs in 860: the drop is the cause of 1,050.
Table cells fall from 584 to 0, headings from 19 to 0, and list items from 71 to 2.
hakkiri keeps each alert at its own place, since each block was measured, and the alert is as true as the others.
Alternative: drop the alert the way Vale does.
It lost because it loses a measurement that exists, and it needs Vale's search to know which blocks collide.

### 7. A link that repeats its address

Vale replaces the text of an element whose address equals its text with asterisks ([clean][v-clean]), so `[a.md](a.md)` counts no word.
hakkiri did not, and now does, in [HKR-064][ledger].
Vale does the same for `<https://a.example>` but not for `<a@b.example>`, whose address is `mailto:a@b.example`, so it counts the three words of the email, and hakkiri does too.

### 8. The sentences that remain

Of the 860 differences left with Vale's check off, 70 are the text of a link that repeats its address, which [HKR-064][ledger] fixes.
The 790 that remain are in how sentences split.
They are the sentence alerts and the counts built on sentences: `sentences`, the scores and the paragraph metrics.
Two of them are the long words of `はっきり`, which `vale-differences` already records as a count of characters.
The other 788 belong to the sentence splitter, row [HKR-065][ledger]: 172 minimal inputs, each still different from Vale, fall into four groups by what ends or starts the sentence.

[engine-text]: ../archive/2026-11-10-rule-engine/design.md
[pattern-9]: ../archive/2026-11-10-pattern-and-readability-checks/design.md#9-code-is-no-match-in-the-pattern-checks
[v-history]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/file.go#L568-L571
[v-advance]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/walk.go#L831-L864
[v-clean]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/ast.go#L744-L747

[ledger]: ../../../tasks.yaml
