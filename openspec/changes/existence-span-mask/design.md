# Design

## Decisions

### 1. A pattern check reads a code span as one `*` and drops a match only at the edges of the match

`run_existence` searched the text where a code span is the letter `x`, and dropped each match that overlapped a span.
Vale masks a span as a run of `*`, and a match that runs across one is a match.
Of the 487 alerts of `ClauseChain` that Vale gave and hakkiri did not, 452 held a span inside the match, and 39 of 50 sampled were real chains.
`Text::pattern_reading` now gives the text with each span as one `*`, a byte that no word and no space matches and that the text already uses for masked markup, so every offset holds.
`Text::edge_in_code` drops a match whose first or last byte lies in a span, and keeps one with text on both sides.
The sequence walk reads tokens from the tagger and keeps decision 2 of `code-span-tokens`, so the two readings do not meet.
`substitution`, `occurrence`, `repetition` and the rest keep the refusal of an overlap.

## Measures

### `ClauseChain`

On the same nine repositories the package's `ClauseChain` gave 1,020 alerts before decision 1 and 1,536 after it, where Vale gives 1,492.
The match text differs on 371 of them, where Vale prints the mask as `*` and hakkiri prints `x`, so the diff counts each as changed.
Pairing the alerts of both tools by file and by line, within 3 lines, leaves 464 alerts that only Vale gave before decision 1 and none after it.
The 13 that remained in the first run of this change were the alerts of the repository whose configuration the run converted wrongly, so no sentence splitter explains them.
It leaves 40 alerts that only hakkiri gives, 11 of them before decision 1.
Two readers labelled them against the rule's definition and differed on none: 26 are true and 14 are false.
The 14 false alerts join the items of a list with semicolons, or follow a label colon with items, so no part holds a verb.
The rule's pattern cannot tell a list from a chain.
Vale gives the same kind of false alert: of the 88 alerts that only Vale gave before decision 1, 46 are false.
The cause lies in the package and not in the engine.
