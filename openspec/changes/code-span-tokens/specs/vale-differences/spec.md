## ADDED Requirements

### Requirement: A code span is a noun of a sequence

hakkiri SHALL read a code span as a noun in a `sequence` rule, and a pattern for a lowercase word SHALL refuse it.  
Vale 3.26.0 reads the span as a run of `*`, and a rule with a tag and no pattern flags it.

#### Scenario: A code name before a verb

- **WHEN** a rule has `tokens: [{tag: 'NN|NNS'}, {pattern: says}]` and the input is ``The call fails as `flag` says.``
- **THEN** hakkiri and Vale both give an alert, and Vale reports `**** says` ([code-span-tokens 1][code-span-tokens-1])

### Requirement: An existence match across a code span is a match

hakkiri SHALL report an `existence` match that runs across a code span, as Vale does, and SHALL drop one that starts or ends on a span.
Vale 3.26.0 masks the span as a run of `*` and reports a match that starts on it as well.
The matched text differs: hakkiri prints `x` for the span, and Vale prints the mask.

#### Scenario: A chain across a code span

- **WHEN** the input is ``The loop reads `a`; the store keeps `b`, which is slow.`` under `ClauseChain`
- **THEN** hakkiri and Vale both give an alert, and the two print the span differently in the matched text ([code-span-tokens 7][code-span-tokens-7])

### Requirement: The nouns of technical prose read as nouns

hakkiri SHALL tag `check`, `split`, `round`, `prompt`, `fade` and `thinking` as nouns after a determiner, in the title of a pull request after a noun, and after two singular nouns before a preposition, where the model tags them as another part of speech.
A rule of four lowercase nouns then counts them, and the verb `check` after a plural noun stays a verb.

#### Scenario: A word of a pull request title

- **WHEN** the input is `The queue load split pull request ended it.` or `It is merged (the prompt template pull request).`
- **THEN** `split` and `prompt` read `NN`, and `final` in `the final mask round pull request` stays `JJ` ([code-span-tokens 8][code-span-tokens-8])

#### Scenario: A verb of the same spelling

- **WHEN** the input is `The tests check the cache against the spec.`
- **THEN** `check` reads `VB`

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
[code-span-tokens-7]: ../../design.md#7-a-pattern-check-reads-a-code-span-as-one--and-drops-a-match-only-at-its-edges
[code-span-tokens-8]: ../../design.md#8-the-tagger-reads-the-nouns-that-technical-prose-uses-where-the-model-reads-another-part-of-speech
