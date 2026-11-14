## ADDED Requirements

### Requirement: The nouns of technical prose read as nouns

hakkiri SHALL tag `check`, `prompt`, `thinking` and `fade` as nouns after a determiner, where the model tags them as another part of speech.
It SHALL tag `prompt`, `round` and `split` as nouns after a noun when the nouns after them run up to `pull request`.
It SHALL tag `check` as a noun when a base form follows two singular nouns before a preposition or a stop.
A rule of four lowercase nouns then counts them, and the verb `check` after a plural noun stays a verb.

#### Scenario: A word of a pull request title

- **WHEN** the input is `The queue load split pull request ended it.` or `It is merged (the prompt template pull request).`
- **THEN** `split` and `prompt` read `NN`, and `final` in `the final mask round pull request` stays `JJ` ([technical-nouns 1][technical-nouns-1])

#### Scenario: A verb of the same spelling

- **WHEN** the input is `The tests check the cache against the spec.`
- **THEN** `check` reads `VB`

[technical-nouns-1]: ../../design.md#1-the-tagger-reads-the-nouns-that-technical-prose-uses-where-the-model-reads-another-part-of-speech
