## ADDED Requirements

### Requirement: A code span is no word of a run

A `sequence` rule SHALL read a code span as a word that no token names, by its `tag`, its `pattern` or its `words`.  
A negated token SHALL hold at it.  
A token that reads one word SHALL instead read the noun that a preposition before the span attaches it to, when there is one, and a run that starts at the span SHALL start at that noun.

#### Scenario: A code span in place of a noun

- **WHEN** a rule has `message: "'%s %s' gives a thing speech."` and `tokens: [{tag: 'NN|NNS'}, {pattern: says}]`, and the input is ``The call fails as `flag` says.``
- **THEN** hakkiri reports no alert ([code-span-tokens 1][code-span-tokens-1])

#### Scenario: A prose noun

- **WHEN** the same rule reads `The call fails as the flag says.`
- **THEN** hakkiri reports one alert, with the matched text `flag says`

#### Scenario: A code span that a preposition attaches to a noun

- **WHEN** the same rule reads ``The notice for `cmd-7` says a person stopped it.``
- **THEN** hakkiri reports one alert, with the matched text `notice for x says` and the message `'notice says' gives a thing speech.` ([code-span-tokens 4][code-span-tokens-4])

#### Scenario: A repeated token

- **WHEN** a rule has `tokens: [{tag: 'NN|NNS', min: 4}]` and the input is ``The session lake store `x` holds it.``
- **THEN** hakkiri reports no alert

#### Scenario: A negated token at a code span

- **WHEN** a rule has `message: "'%s' ends."` and `tokens: [{pattern: files}, {tag: NN, negate: true}]`, and the input is ``Copy the files `data` here.``
- **THEN** hakkiri reports one alert, with the message `'files' ends.` ([code-span-tokens 2][code-span-tokens-2])

[code-span-tokens-1]: ../../design.md#1-the-walk-refuses-the-word
[code-span-tokens-2]: ../../design.md#2-a-negated-token-holds-at-a-code-span
[code-span-tokens-4]: ../../design.md#4-a-preposition-attaches-the-span-to-a-prose-noun
