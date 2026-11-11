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

[engine-text]: ../archive/2026-11-10-rule-engine/design.md
[pattern-9]: ../archive/2026-11-10-pattern-and-readability-checks/design.md#9-code-is-no-match-in-the-pattern-checks
