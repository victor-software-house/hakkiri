# Design

## Decisions

### 1. A pattern check reads a code span as one `*` and drops a match only at the edges of the match

`run_existence` searched the text with each code span as the letter `x`.  
It dropped each match that overlapped a span.

Vale masks a span as a run of `*`, and a match that runs across one is a match.  
Vale gave 487 alerts of `ClauseChain` that hakkiri did not.  
Of these, 452 had a span inside the match, and 39 of 50 sampled were real chains.

`Text::pattern_reading` now gives the text with each span as one `*`.  
No word and no space matches that byte, and the text already uses it for masked markup, so every offset stays the same.

`Text::edge_in_code` drops a match whose first or last byte lies in a span, and keeps one with text on both sides.  
The sequence walk reads tokens from the tagger and keeps decision 2 of `code-span-tokens`, so the two readings do not meet.  
`substitution`, `occurrence`, `repetition` and the rest still reject a match that overlaps a span.

## Measures

### `ClauseChain`

On the same nine repositories, the package's `ClauseChain` gave 1,020 alerts before decision 1 and 1,536 after it.  
Vale gives 1,492.  
The match text differs on 371 of them: Vale prints the mask as `*` and hakkiri prints `x`.  
So the diff counts each of these as changed.

We paired the alerts of both tools by file and line, within 3 lines.  
Before decision 1, this left 464 alerts that only Vale gave, and after it none.  
The first run of this change left 13.  
They came from the one repository whose configuration the run converted wrongly, so no sentence splitter explains them.

The pairing leaves 40 alerts that only hakkiri gives, 11 of them before decision 1.  
Two readers labelled them against the rule's definition and differed on none: 26 are true and 14 are false.  
The 14 false alerts join list items with semicolons, or put items after a label and a colon.  
So no part has a verb.

The rule's pattern cannot tell a list from a chain.  
Vale gives the same kind of false alert: of the 88 alerts that only Vale gave before decision 1, 46 are false.  
The cause lies in the package, not in the engine.
