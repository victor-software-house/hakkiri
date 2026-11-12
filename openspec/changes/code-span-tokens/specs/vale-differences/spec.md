## ADDED Requirements

### Requirement: A code span is a noun of a sequence

hakkiri SHALL read a code span as a noun in a `sequence` rule, and a pattern for a lowercase word SHALL refuse it.  
Vale 3.26.0 reads the span as a run of `*`, and a rule with a tag and no pattern flags it.

#### Scenario: A code name before a verb

- **WHEN** a rule has `tokens: [{tag: 'NN|NNS'}, {pattern: says}]` and the input is ``The call fails as `flag` says.``
- **THEN** hakkiri and Vale both give an alert, and Vale reports `**** says` ([code-span-tokens 1][code-span-tokens-1])

### Requirement: The verbs of saying and asking keep their place

hakkiri SHALL tag a verb of saying or asking after a noun as the verb it is, and `asks` and `asked` in the noun phrase of a preposition as a noun and a participle, so a `sequence` rule with a verb tag flags the first and no other.  
Vale tags the same words with the older model, and its rule has no verb tag.

#### Scenario: A verb after a noun

- **WHEN** the input is `The docs say so.`, `The CI builds agree with each other.` or `cargo asks again with a header.`
- **THEN** `say` and `agree` read as verbs, `builds` reads `NNS`, and `cargo` reads `NN` ([code-span-tokens 6][code-span-tokens-6])

#### Scenario: A noun and a participle

- **WHEN** the input is `Answering the name asks that wait.` or `with the cache hook told of the new file`
- **THEN** `asks` reads `NNS`, and `told` reads `VBN`, so a rule with the tag `^(?:VB|VBD|VBP|VBZ)$` on its verb flags neither ([code-span-tokens 6][code-span-tokens-6])

[code-span-tokens-1]: ../../design.md#1-the-model-reads-a-noun-in-place-of-a-code-span
[code-span-tokens-6]: ../../design.md#6-the-verb-token-of-thingsspeaknoun-carries-a-tag-and-the-tagger-corrections-make-it-hold
