## ADDED Requirements

### Requirement: A code span counts as code_span_size words

A `metric` or `readability` rule SHALL count each code span of the text it measures as `code_span_size` words: one by default.  
At 0 a code span SHALL be no word, and at 2 or more it SHALL be that many words.  
The count of `occurrences` SHALL not change with the size.

#### Scenario: Sizes of a sentence

- **WHEN** a rule has `extends: metric`, `scope: sentence`, `formula: words` and `condition: '> 2'`, and the input is ``Run `a`, `b`, `c` and `one two three` today.``
- **THEN** hakkiri reads 7.00 at size 1, 3.00 at size 0 and 11.00 at size 2 ([code-span-size 4][code-span-size-4])

#### Scenario: A file with no scope

- **WHEN** the rule has no `scope`
- **THEN** it reads the same counts of the prose of the whole file

[code-span-size-4]: ../../design.md#4-it-reaches-word-counts-only
