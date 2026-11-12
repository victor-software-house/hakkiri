## ADDED Requirements

### Requirement: A code span is a noun of a sequence

hakkiri SHALL read a code span as a noun in a `sequence` rule, and a pattern for a lowercase word SHALL refuse it.  
Vale 3.26.0 reads the span as a run of `*`, and a rule with a tag and no pattern flags it.

#### Scenario: A code name before a verb

- **WHEN** a rule has `tokens: [{tag: 'NN|NNS'}, {pattern: says}]` and the input is ``The call fails as `flag` says.``
- **THEN** hakkiri and Vale both give an alert, and Vale reports `**** says` ([code-span-tokens 1][code-span-tokens-1])

[code-span-tokens-1]: ../../design.md#1-the-model-reads-a-noun-in-place-of-a-code-span
