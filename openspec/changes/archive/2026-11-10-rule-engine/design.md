# Design

## Context

The motivation is in [proposal.md][proposal].  
The tree is built ([document-tree][tree-spec], [node-segments][segments-spec]); the rule format is fixed ([rule-files][rule-files], [rule-format design][format]).  
Every Vale behaviour below was read in Vale's source at [`66bbb85`][vale], which is Vale 3.26.0 plus 16 commits.  
Vale's counts come from the [prose][prose] module it pins, read at `b33e60d`.

## Goals / Non-Goals

**Goals:**

- One engine that runs a rule set over one file's tree and returns alerts with file positions.
- Vale 3.26.0's meaning for every field, except the intended differences in [rule-format decision 7][format-7].
- Linear work in the size of the file, so long lines and long files finish.

**Non-Goals:**

- The command line, output formats and exit status: `HKR-005`.
- Packages and project config: `HKR-006` and `HKR-005`.
- Vocabularies. hakkiri has no vocabulary file, so `vocab` loads and has no effect.
- Fixes. `action` loads and rides on the alert; nothing applies it until `HKR-011`.
- Vale's `doc(...)` selectors and Views. They are not Vale scope names in the format, so they are refused as unknown scopes.
- `affix`, which needs the spelling change's dictionaries: until [`case-sequence-and-spelling-checks`][last], a rule that sets it is refused.

## Decisions

### 1. One entry point per file

The engine takes one file's tree, its source text, its extension and the loaded rules, and returns that file's alerts.  
Per-file state, such as the alert history, lives in one value made for that call.

Alternative: one engine value for a whole run that keeps state across files. Lost: the language server in release 2 lints one file at a time ([bootstrap decision 6][bootstrap]).

### 2. Rules decode into one type per check

Each check type has its own rule type, decoded with unknown fields denied, as Vale's decoder denies them ([definition.go][v-decode]).  
Patterns, formulas and conditions compile at load, so a broken rule fails before any file is read.

Alternative: keep each rule as a map and read fields while linting, as Vale's `baseCheck` does. Lost: a wrong field would surface only in a file that reaches it.

### 3. A scope part matches by dotted prefix

`heading` selects `heading.h1` to `heading.h6`, and `text.comment` selects all four comment scopes.  
Vale compares the set of dotted parts instead ([scope.go][v-contains]), so `h1` alone or `comment.text` would also match.

Alternative: Vale's set test. Lost: the [rule-files spec][rule-files] refuses a name it does not define, and the set test accepts names no node holds. Every scope ProseStyle writes reads the same under both.

### 4. text and paragraph keep Vale's blocks

Vale makes one block for each paragraph, list item, quote and table cell ([ast.go][v-blocks]).  
Only a paragraph outside a list item or quote is a `paragraph` ([ast.go][v-paragraph]).  
hakkiri's tree nests paragraphs inside list items and quotes.  
So `text` selects paragraphs, headings and table cells, and `paragraph` selects a paragraph with no list item or quote above it.

Alternative: `paragraph` selects every paragraph node. Lost: ProseStyle's `Paragraph` metric would then measure list items, a difference the format design does not list.

### 5. A check reads text built from segments

Each prose node's check text is built once per file from its segments.  
A table maps each byte of the check text to its byte in the file.  
Inline markup is a gap (the [node-segments delta][segments-delta]).  
So is an autolink or a bare URL, which counts no words in Vale ([rule-format decision 7][format-7]).

A code span becomes the one-letter word `x`, so it counts as one word, as the format decides.  
An `existence` or `substitution` match that overlaps that word gives no alert.  
An `occurrence` match that overlaps it does not count.  
An inline exclusion such as `~link` turns each character of the element into spaces of the same byte width.  
Vale does the same ([provider.go][v-without]), and offsets hold.

Alternatives:

- Vale's mask, which writes code as `*` and counts no word. Lost: the format counts one word.
- Keep the code text and skip matches inside it. Lost: a word count would count every word of the code.

### 6. Sentences per prose leaf, words by prose's pattern

A sentence is a piece of one paragraph, heading or table cell, cut by hakkiri's own splitter ([sentence-splitter change][splitter]).  
Vale segments the same blocks ([provider.go][v-segment]).

A whole-file metric reads the paragraphs, list items and quotes joined by blank lines, as Vale's summary does ([ast.go][v-summary]).  
In a code file it reads the paragraphs of the comments, which Vale's default does not (decision 16).  
It then cuts at blank lines and sentences ([summarize.go][p-init]).  
A word is a match of `(?:[A-Z]\.){2,}|[\p{N}\p{L}']+`, prose's word pattern ([regexp.go][p-words]).

Alternative: count words by whitespace. Lost: `characters` and syllables are counted per word, so the words must be Vale's.

### 7. Positions from one line table

The engine builds one table of line starts per file and finds a line by binary search.  
A column counts characters from the line start, 1-based, and the last column is inclusive, as in Vale ([file.go][v-columns]).

The last column is where the match's last character lies in the file.  
Markup inside a match widens it, as in Vale's placement by byte offsets.  
A match that goes on to the next line ends at the end of its first line.

Alternative: find the match by searching for its text, as Vale's default path does ([file.go][v-search]). Lost: a repeated phrase lands on the wrong copy, and the search is quadratic on a long paragraph.

### 8. Two regex engines and a backtrack limit

A pattern compiles with `regex` 1.13.1.  
When `regex` refuses it for lookaround or a backreference, it compiles with `fancy-regex` 0.20.1 and its backtrack limit ([bootstrap decision 8][bootstrap]).  
Both are MIT or Apache-2.0, which [deny.toml][deny] allows.  
A limit reached gives one `error` alert for that rule and file.

Word and case flags follow Vale's templates: `(?m)\b(?:…)\b`, `(?m)(?:…)` with `nonword`, and `(?i)` with `ignorecase` ([definition.go][v-templates]).

Vale runs every pattern on regexp2 in RE2 mode with no time limit, and panics when a match fails ([regex.go][v-regex]).

Alternative: `fancy-regex` for every pattern, since it hands simple ones to `regex`. Lost: every search then returns a `Result`, and a test cannot see which engine a pattern took.

### 9. Messages, order and duplicates as in Vale

`%s` takes the values in order and `%%` is `%`, as Vale's `CondSprintf` does ([util.go][v-format]).  
A `%s` with no value left reads as nothing, where Go prints `%!s(MISSING)`.  
An `occurrence` message reads `%d` as `%s` before the count fills it, as Vale does ([occurrence.go][v-occ-format]).

Alerts sort by line and column, stable in rule order ([file.go][v-sort]).  
An alert with the rule, line and column of an earlier one is dropped, and `limit` caps a rule per file ([file.go][v-history]).

Alternative: keep duplicates. Lost: `scope: [list, blockquote]` on a list in a quote would report each match twice.

### 10. Metric variables and conditions as in Vale

The variables are Vale's ([file.go][v-metrics], [metric.go][v-metric]), and the counts and scores are prose's ([summarize.go][p-init], [readability.go][p-scores]).  
Element counts come from the tree; `pre` reads the tree's code block count (the [document-tree delta][tree-delta]).  
The condition runs as the result, printed with six decimals, followed by the condition, with `result` bound, as Vale builds it ([metric.go][v-condition]).

Alternative: compare the unrounded result. Lost: a result just above a threshold would alert where Vale does not.

### 11. Metric tokens match as existence tokens

A metric's `tokens` take word boundaries unless `nonword` is set, as in `existence`.  
The [rule-files delta][rule-files-delta] adds `nonword: true` to the semicolon scenario, which counted 2 only without boundaries.

Alternative: read metric tokens as written, as `occurrence` reads `token`. Lost: one field name with two meanings.

### 12. Flesch-Kincaid parity has two tolerances

The parity fixture holds Vale's `flesch_kincaid`, `words`, `sentences` and `syllables` for each prose-style case input with no code span.  
Inputs with a code span stay out, since a code span counts one word on purpose.

Where hakkiri's three counts equal Vale's, its score is within 0.01 of Vale's.  
On every input it is within 0.5 of a grade level, and the test prints each input whose counts differ.

Alternatives:

- One tolerance of 0.5 everywhere. Lost: a syllable bug and a sentence bug that cancel would pass.
- Exact scores. Lost: hakkiri's splitter cuts a few inputs apart from Vale, where it splits better.

### 13. Rule files load through ctl-core

A rule file is read by the `input` feature of [ctl-core][ctl-input] (operator, 2026-11-10).  
It parses YAML 1.2 with serde-saphyr, validates once with garde, and places each problem on its file and line through yamled.  
The ctl CLIs read their config the same way, so hakkiri adds no YAML crate of its own.

Alternative: call serde-saphyr or another YAML crate directly. Lost: hakkiri would repeat the error placement and size limits that ctl-core already owns.

### 14. Formulas run on an own parser

A formula is parsed by hakkiri's own code, about 300 lines, for the grammar in the [metric spec][metric-spec] (operator, 2026-11-10).  
It reads `math.sqrt` and `round` as Tengo writes them.

Alternatives:

- `rhai`: a whole scripting language, whose module calls are written `math::sqrt`.
- `exmex`: each Tengo `math` name needs its own entry in code.
- `evalexpr`: AGPL-3.0, which [deny.toml][deny] refuses.

### 15. Readability counts are a port of prose

Syllables, complex words and the Dale-Chall list are ported from prose with its MIT notice (operator, 2026-11-10).  
Syllables follow prose's port of the `syllable` JavaScript package ([syllables.go][p-syllables]).  
A complex word drops one syllable for `es`, `ed` or `ing` ([summarize.go][p-complex]).  
The Dale-Chall list is prose's 2,944 easy words, and each easy word counts once ([easy.go][p-easy], [readability.go][p-scores]).

Alternatives: `syllarust`, `syllable` and `hyphenation` count syllables apart from prose, and `dale-chall` does not state its source list. Lost with each: Flesch-Kincaid would drift from Vale's for no gain.

### 16. Better than Vale where Vale is wrong

hakkiri gives at least Vale's result, and a better one where Vale's behaviour is a defect (operator, 2026-11-10).  
These rows join the intended differences of [rule-format decision 7][format-7]:

| Case | Vale 3.26.0 | hakkiri |
|:--|:--|:--|
| `\w`, `\d` and `\s` in a pattern | ASCII, while `\b` is Unicode ([regexp2 README][re2-mode]) | Unicode, as `\b` is, so `naïve` is one word |
| Where a measurement alert points | Column 1 of its block's first line ([file.go][v-measure]) | The first byte of the measured node, so a list item points past its bullet |
| A long word | Over six bytes ([summarize.go][p-complex]) | Over six characters |
| `capitalize` on a match that starts with a digit or a symbol | Capitalizes the value, since changing the first character leaves the match unchanged ([substitution.go][v-capitalize]) | Capitalizes only when the match starts with a capital letter |
| `$1` in a swap value, for a key with a space | The key as written misses a line wrap, so the alert is lost ([substitution.go][v-expand]) | The key as matched, its spaces matching a line wrap |
| `occurrence` with matches of blank text only | Counts them, points at none, and gives no alert ([occurrence.go][v-occ-count]) | Points at the first word of the node |
| `occurrence` over a match in code | Counts every match, then points at the first one that is not code ([occurrence.go][v-occ-count]) | A match that overlaps a code span is not counted |
| A whole-file metric in a code file | Prints no value (operator, 2026-11-10). A comment joins the summary only when a `tree-sitter` View gives its query a markup format ([code_ts.go][v-code-summary]). The default Rust query has none ([rs.go][v-rust-query], [query.go][v-query-format]), so `lintLines` writes no summary ([code_ts.go][v-comment-lint], [ast.go][v-summarize]) | Measures the prose of the comments, built in |

Alternative: Vale's behaviour in each case.  
Lost: a word with an accent would read as two words, an alert would point at a bullet, and `1st` would swap to `First`.  
A `$1` key would lose its alert across a line wrap, and a count would include code.

The comments row is a choice, not a repair of a Vale defect (operator, 2026-11-10).  
The same holds at Vale 3.26.0 ([e2d4374][v-3-26-0]).  
Alternative: match Vale's empty summary for a code file.  
Lost: hakkiri measures comment prose by default, and the list of languages it reads will grow.  
The [parity test][parity] asserts it on the 10 Rust inputs.

### 17. An inline exclusion belongs to its scope item

`~link`, `~code`, `~strong` and `~emphasis` blank an element for the blocks that the item holding them selects, and for no other block.  
With `scope: [heading, 'paragraph & ~link']`, a link in a heading still reads as text, and a link in a paragraph reads as spaces.  
When two items select one block, it reads with the exclusions of both.

Vale gathers the exclusions of every item into one list, `Excluded`, and blanks them in every block the rule reads ([scope.go][v-excluded-list], [lint.go][v-excluded-use]).  
That is a defect: an exclusion written for one item changes the text of another.

Alternative: Vale's one list for the rule. Lost: `[heading, 'paragraph & ~link']` would also blank the links in headings, and a rule could not exclude an element in one scope only.

### 18. Every phrase exception holds, overlapping or nested

A match is held back when it lies inside any match of any multi-word exception.  
That holds when two phrases overlap, such as `a b` and `b c` in `a b c`, and when one holds the other, such as `a b` and `a b c`.  
Each phrase is searched on its own, and each search restarts one character past the start of its last match.

Vale joins the phrases into one alternation, in list order, and reads its non-overlapping matches ([definition.go][v-phrases], [definition.go][v-within]).  
In `a b c`, `b c` is never found after `a b`, and with `a b` ahead of `a b c`, the alternation stops at `a b`.  
Vale then reports the `c`, in the first case and in the second when the list names `a b` first.

Alternative: Vale's one alternation. Lost: an accepted phrase would still raise an alert when it overlaps another, or follows a shorter one in the list.

### Crates read

Each crate was read on crates.io on 2026-11-10.

| Crate | Version | Released | License | deny.toml |
|:--|:--|:--|:--|:--|
| `regex` | 1.13.1 | 2026-07-15 | MIT OR Apache-2.0 | allowed |
| `fancy-regex` | 0.20.1 | 2026-11-09 | MIT | allowed |
| `ctl-core` | 0.7.0 | 2026-11-09 | MIT | allowed |

## Risks / Trade-offs

- [hakkiri's splitter cuts some text apart from Vale's] → `LongSentence` and Flesch-Kincaid move on those inputs.  
  The parity fixture states the tolerance, and `HKR-008` diffs the alerts.
- [A code span reads as `x`] → a pattern that matches one letter counts it. Match checks drop such matches; counts keep them, as the format intends.
- [fancy-regex may not reach its limit on every slow pattern] → the backtrack test pins one pattern that does.  
  The 50,000-character line test runs every check.
- [Inline gaps change every tree snapshot] → the snapshots are reviewed in the same pull request.

[proposal]: proposal.md
[metric-spec]: specs/metric-check/spec.md
[segments-delta]: specs/node-segments/spec.md
[tree-delta]: specs/document-tree/spec.md
[rule-files-delta]: specs/rule-files/spec.md
[last]: ../../case-sequence-and-spelling-checks/proposal.md
[splitter]: ../2026-11-10-sentence-splitter/design.md
[tree-spec]: ../../../specs/document-tree/spec.md
[segments-spec]: ../../../specs/node-segments/spec.md
[rule-files]: ../../../specs/rule-files/spec.md
[format]: ../2026-11-10-rule-format/design.md
[format-7]: ../2026-11-10-rule-format/design.md#7-intended-differences-from-vale
[bootstrap]: ../2026-11-09-bootstrap-hakkiri/design.md
[deny]: ../../../../deny.toml
[ctl-input]: https://github.com/victor-software-house/ctl-core/blob/883b436ea387df2a0df1c8586373f6def2de6ea9/ctl-core/src/input.rs
[vale]: https://github.com/vale-cli/vale/tree/66bbb85a1a78a5b5497331e04d4033357caced4d
[prose]: https://github.com/jdkato/prose/tree/b33e60dbff0492289b227f3681120ac057d8131b
[v-decode]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/definition.go#L556-L570
[v-templates]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/definition.go#L185-L190
[v-contains]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/scope.go#L191-L194
[v-blocks]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/ast.go#L51-L58
[v-paragraph]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/ast.go#L396-L461
[v-summary]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/ast.go#L465-L472
[v-excluded-list]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/scope.go#L60-L62
[v-excluded-use]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/lint.go#L781-L785
[v-without]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/nlp/provider.go#L81-L100
[v-segment]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/nlp/provider.go#L322-L357
[v-columns]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/file.go#L505-L551
[v-search]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/file.go#L612-L651
[v-sort]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/file.go#L343-L353
[v-history]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/file.go#L667-L685
[v-capitalize]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/substitution.go#L500-L506
[v-phrases]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/definition.go#L517-L539
[v-within]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/definition.go#L541-L555
[v-expand]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/substitution.go#L516-L522
[v-occ-count]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/occurrence.go#L63-L109
[v-code-summary]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/code_ts.go#L112-L114
[v-comment-lint]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/code_ts.go#L199-L215
[v-rust-query]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/code/rs.go#L11-L16
[v-query-format]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/code/query.go#L166
[v-summarize]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/ast.go#L464-L481
[v-3-26-0]: https://github.com/vale-cli/vale/tree/e2d4374b1c66c8a1dd6d912bebc135040d8833fa
[parity]: ../../../../tests/parity.rs
[v-measure]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/file.go#L701-L729
[v-metrics]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/file.go#L362-L405
[v-metric]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/metric.go#L19-L121
[v-condition]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/metric.go#L99-L106
[v-occ-format]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/occurrence.go#L125-L126
[v-format]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/util.go#L154-L163
[v-regex]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/regex/regex.go#L1-L68
[p-init]: https://github.com/jdkato/prose/blob/b33e60dbff0492289b227f3681120ac057d8131b/summarize/summarize.go#L110-L148
[p-complex]: https://github.com/jdkato/prose/blob/b33e60dbff0492289b227f3681120ac057d8131b/summarize/summarize.go#L226-L241
[p-scores]: https://github.com/jdkato/prose/blob/b33e60dbff0492289b227f3681120ac057d8131b/summarize/readability.go#L7-L71
[p-syllables]: https://github.com/jdkato/prose/blob/b33e60dbff0492289b227f3681120ac057d8131b/summarize/syllables.go#L11-L45
[p-easy]: https://github.com/jdkato/prose/blob/b33e60dbff0492289b227f3681120ac057d8131b/summarize/easy.go
[p-words]: https://github.com/jdkato/prose/blob/b33e60dbff0492289b227f3681120ac057d8131b/tokenize/regexp.go#L62-L66
[re2-mode]: https://github.com/jdkato/regexp2/blob/af31cb0c57f76300c511ddf59009a2b27c2ade4b/README.md#L276-L283
