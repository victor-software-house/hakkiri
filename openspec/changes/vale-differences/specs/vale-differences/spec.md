# vale-differences Specification

## Purpose

This spec lists every intended difference between hakkiri and Vale 3.26.0.  
Each scenario names the case, what Vale does, what hakkiri does, and links the decision that records it.  
A known gap, where hakkiri lacks something Vale does and means to close it, is not a difference.  
It is listed apart, with its queue row, under Known gaps.

## ADDED Requirements

### Requirement: Every intended difference is listed here

Every intended difference from Vale 3.26.0 SHALL be a scenario of this spec, with a link to the decision that records it.  
A pull request that adds, changes or removes a difference SHALL change this spec in the same pull request.

#### Scenario: A change adds a difference

- **WHEN** a change decides that hakkiri reports, refuses or measures something otherwise than Vale does
- **THEN** the same pull request adds a scenario for it to this spec, and links its decision

#### Scenario: A later decision removes a difference

- **WHEN** a decision makes hakkiri match Vale where an earlier decision differed, as [case 9][case-9] did for the `~` before a scope that [format 7][format-7] listed
- **THEN** the same pull request removes or rewrites the scenario, so the list holds only what is true now

### Requirement: What a rule reads differs from Vale

hakkiri SHALL read the text and the scopes of a document as each scenario below says, where Vale 3.26.0 reads them otherwise.

#### Scenario: A code span counts one word

- **WHEN** a sentence of two words, a four-word code span and one more word is measured
- **THEN** hakkiri counts 4 words and Vale counts 3 ([format 7][format-7])

#### Scenario: Rust block comments are read

- **WHEN** a Rust file holds a `/* */`, `/** */` or `/*! */` comment
- **THEN** hakkiri reads it as `text.comment.block` or `text.comment.doc.block`, and Vale reads line comments only ([format 7][format-7])

#### Scenario: String literals are never prose

- **WHEN** a code file holds a string literal
- **THEN** hakkiri never reads it as prose, and Vale reads it with the `raw` scope ([format 7][format-7])

#### Scenario: A scope part matches by dotted prefix

- **WHEN** a rule has `scope: h1` or `scope: comment.text`
- **THEN** hakkiri refuses the name as unknown, and Vale compares the set of dotted parts, so it selects the nodes that hold them ([engine 3][engine-3])

#### Scenario: A sentence in a paragraph is selected

- **WHEN** a rule has `scope: sentence & paragraph`
- **THEN** hakkiri selects the sentences of paragraphs, and Vale selects none ([case 9][case-9])

#### Scenario: A negated paragraph is left out

- **WHEN** a rule has `scope: ~paragraph`
- **THEN** hakkiri leaves paragraphs out, and Vale leaves out nothing, so hakkiri gives 104 alerts fewer on the nine files of the scope test ([case 9][case-9])

#### Scenario: An inline exclusion belongs to its scope item

- **WHEN** a rule has `scope: [heading, 'paragraph & ~link']`
- **THEN** a link in a heading still reads as text in hakkiri, and Vale blanks it, since it gathers the exclusions of every item into one list ([engine 17][engine-17])

#### Scenario: Sentences are cut by hakkiri's splitter

- **WHEN** a text has a break that the two splitters cut differently
- **THEN** hakkiri cuts it by its own splitter, which is held to split at least as well as Vale on every evaluation set, so a few inputs read apart from Vale ([splitter 5][splitter-5], [engine 12][engine-12])

### Requirement: Where an alert points and what it matches differs from Vale

hakkiri SHALL place an alert, and match text for `existence`, `substitution` and `occurrence`, as each scenario below says, where Vale 3.26.0 does otherwise.

#### Scenario: An alert is placed by offset

- **WHEN** a phrase appears twice in a block and the second copy matches
- **THEN** hakkiri points at the second copy through its byte table, and Vale's default path searches for the matched text, so it can land on the first ([engine 7][engine-7])

#### Scenario: A measurement alert points at its node

- **WHEN** a measurement alert fires on a list item
- **THEN** hakkiri points at the first byte of the measured node, past its bullet, and Vale points at column 1 of the block's first line ([engine 16][engine-16])

#### Scenario: Word and space classes are Unicode

- **WHEN** a pattern uses `\w`, `\d` or `\s` on `naïve`
- **THEN** hakkiri reads one word, since these classes are Unicode as `\b` is, and Vale reads them as ASCII, so a word with an accent reads as two ([engine 16][engine-16])

#### Scenario: capitalize needs a capital to start

- **WHEN** a swap value is capitalized for a match that starts with a digit or a symbol, such as `1st`
- **THEN** hakkiri capitalizes only a match that starts with a capital letter, and Vale capitalizes the value, so `1st` swaps to `First` ([engine 16][engine-16])

#### Scenario: A swap key with a space reads a line wrap

- **WHEN** a `substitution` key holds a space and `$1` in its value, and the matched text wraps a line
- **THEN** hakkiri expands `$1` from the key as matched, and Vale expands it from the key as written, so the alert is lost ([engine 16][engine-16])

#### Scenario: An occurrence over blank text points at a word

- **WHEN** an `occurrence` rule matches blank text only
- **THEN** hakkiri points at the first word of the node, and Vale counts the matches, points at none and gives no alert ([engine 16][engine-16])

#### Scenario: An occurrence over code is not counted

- **WHEN** an `occurrence` match overlaps a code span
- **THEN** hakkiri does not count it, and Vale counts every match, then points at the first one that is not code ([engine 16][engine-16])

#### Scenario: Overlapping phrase exceptions both hold

- **WHEN** the text is `a b c`, the exceptions are the phrases `a b` and `b c`, and a rule matches `c`
- **THEN** hakkiri holds back the match, and Vale reports `c`, since its one alternation never finds `b c` after `a b` ([engine 18][engine-18])

### Requirement: Measurements differ from Vale

hakkiri SHALL measure as each scenario below says, where Vale 3.26.0 measures otherwise.

#### Scenario: A long word is over six characters

- **WHEN** a word has six characters and seven bytes, such as `naïves`
- **THEN** hakkiri does not count it as a long word, and Vale counts every word over six bytes ([engine 16][engine-16])

#### Scenario: A whole-file metric reads code comments

- **WHEN** a whole-file metric runs on a Rust file
- **THEN** hakkiri measures the prose of the comments, and Vale prints no value, since its default Rust query gives no comment a markup format ([engine 16][engine-16])

#### Scenario: A score listed twice counts once

- **WHEN** a `readability` rule has `metrics: [SMOG, SMOG]`
- **THEN** hakkiri averages the distinct scores and reports SMOG's grade, and Vale divides by the length of the list and reports half of it ([pattern 8][pattern-8])

[case-9]: ../../../archive/2026-11-10-case-and-sequence-checks/design.md#9-a-block-of-text-has-the-scope-of-one-element
[engine-12]: ../../../archive/2026-11-10-rule-engine/design.md#12-flesch-kincaid-parity-has-two-tolerances
[engine-16]: ../../../archive/2026-11-10-rule-engine/design.md#16-better-than-vale-where-vale-is-wrong
[engine-17]: ../../../archive/2026-11-10-rule-engine/design.md#17-an-inline-exclusion-belongs-to-its-scope-item
[engine-18]: ../../../archive/2026-11-10-rule-engine/design.md#18-every-phrase-exception-holds-overlapping-or-nested
[engine-3]: ../../../archive/2026-11-10-rule-engine/design.md#3-a-scope-part-matches-by-dotted-prefix
[engine-7]: ../../../archive/2026-11-10-rule-engine/design.md#7-positions-from-one-line-table
[format-7]: ../../../archive/2026-11-10-rule-format/design.md#7-intended-differences-from-vale
[pattern-8]: ../../../archive/2026-11-10-pattern-and-readability-checks/design.md#8-a-score-listed-twice-counts-once
[splitter-5]: ../../../archive/2026-11-10-sentence-splitter/design.md#5-evaluation-against-gold-boundaries-not-against-vale
