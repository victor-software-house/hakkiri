## ADDED Requirements

### Requirement: A code span is a noun that reads as one capital letter

A `sequence` rule SHALL read a code span as a noun: its tag is `NN`, and the tagger reads a noun in its place.  
A pattern or a word list SHALL read the span as the letter `X`.  
The square brackets of an intra-doc link around one code span SHALL belong to the span.

#### Scenario: A code span in place of a noun

- **WHEN** a rule has `message: "'%s %s' gives a thing speech."` and `tokens: [{tag: 'NN|NNS'}, {pattern: says}]`, and the input is ``The call fails as `flag` says.``
- **THEN** hakkiri reports one alert, with the matched text `x says` ([code-span-tokens 1][code-span-tokens-1])

#### Scenario: A prose noun

- **WHEN** the same rule reads `The call fails as the flag says.`
- **THEN** hakkiri reports one alert, with the matched text `flag says`

#### Scenario: An intra-doc link

- **WHEN** the same rule reads ``The call fails as [`flag`] says.``
- **THEN** hakkiri reports one alert, with the matched text `[x] says` ([code-span-tokens 3][code-span-tokens-3])

#### Scenario: A lowercase pattern

- **WHEN** a rule has `tokens: [{tag: 'NN|NNS', pattern: '[a-z][a-z-]*', min: 4}]` and the input is ``The session lake `store` path is long.``
- **THEN** hakkiri reports no alert, and it reports one for `The session lake store path is long.` ([code-span-tokens 2][code-span-tokens-2])

[code-span-tokens-1]: ../../design.md#1-the-model-reads-a-noun-in-place-of-a-code-span
[code-span-tokens-2]: ../../design.md#2-a-pattern-reads-a-code-span-as-x
[code-span-tokens-3]: ../../design.md#3-the-brackets-of-an-intra-doc-link-belong-to-the-span
