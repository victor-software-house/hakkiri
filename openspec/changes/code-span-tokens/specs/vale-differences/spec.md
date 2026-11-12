## ADDED Requirements

### Requirement: A code span is not a word of a sequence

hakkiri SHALL give no alert for a `sequence` rule whose token reads a code span as a word, where Vale 3.26.0 reads the span as a run of `*` that a tag or a pattern can match.

#### Scenario: A code name before a verb

- **WHEN** a rule has `tokens: [{tag: 'NN|NNS'}, {pattern: says}]` and the input is ``The call fails as `flag` says.``
- **THEN** hakkiri gives no alert, and Vale gives one for `**** says` ([code-span-tokens 1][code-span-tokens-1])

[code-span-tokens-1]: ../../design.md#1-the-walk-refuses-the-word-not-the-match
