## ADDED Requirements

### Requirement: A code span is no word of a run

A `sequence` rule SHALL read a code span as a word that no token names, by its `tag`, its `pattern` or its `words`, so no run holds one.  
A negated token SHALL hold at it.

#### Scenario: A code span in place of a noun

- **WHEN** a rule has `message: "'%s %s' gives a thing speech."` and `tokens: [{tag: 'NN|NNS'}, {pattern: says}]`, and the input is ``The call fails as `flag` says.``
- **THEN** hakkiri reports no alert ([code-span-tokens 1][code-span-tokens-1])

#### Scenario: A prose noun

- **WHEN** the same rule reads `The call fails as the flag says.`
- **THEN** hakkiri reports one alert, with the matched text `flag says`

#### Scenario: A negated token at a code span

- **WHEN** a rule has `message: "'%s' ends."` and `tokens: [{pattern: files}, {tag: NN, negate: true}]`, and the input is ``Copy the files `data` here.``
- **THEN** hakkiri reports one alert, with the message `'files' ends.` ([code-span-tokens 2][code-span-tokens-2])

[code-span-tokens-1]: ../../design.md#1-the-walk-refuses-the-word-not-the-match
[code-span-tokens-2]: ../../design.md#2-a-negated-token-holds-at-a-code-span
