## ADDED Requirements

### Requirement: Tags come from the model and its corrections

hakkiri SHALL tag a sentence with the English model of prose at `b33e60d` and then correct the tags by the rules below.
Vale 3.26.0 tags with the model of prose v3.2.2, so the tags of the two tools differ.

#### Scenario: Another model

- **WHEN** both tools tag the 56,516 tokens of UD English GUM
- **THEN** hakkiri's tags match the gold tags 92.53% of the time, and Vale's 90.26% ([tag-corrections 1][tag-corrections-1], [tag-corrections 2][tag-corrections-2])

#### Scenario: A title is tagged by the model alone

- **WHEN** the capitalization check tags a title
- **THEN** hakkiri applies none of the corrections, as Vale applies none ([tag-corrections 4][tag-corrections-4])

### Requirement: Corrections give the tags that documents need

hakkiri SHALL apply to the tags of a sentence the corrections below, in this order, each reading the corrected tag before its word.

#### Scenario: A modal in capitals

- **WHEN** a sentence is `The loop SHALL keep its context.`
- **THEN** `SHALL` reads `MD`, and Vale reads `NNP` ([tag-corrections 2][tag-corrections-2])

#### Scenario: A name in capitals

- **WHEN** a sentence is `The CSS file reads the SSH keys at UTC.`
- **THEN** `CSS`, `SSH` and `UTC` read `NNP`
- **AND** in `THE ANSWER AND THE QUESTION.`, a function word keeps the tag of the model, as does `MB` in `Read the first 10 MB of the file.` ([tag-corrections 5][tag-corrections-5])

#### Scenario: A possessive

- **WHEN** a sentence is `The checkpoint's pages per window show that cost.`
- **THEN** `'s` reads `POS`, and the model alone reads `VBZ`, `NNS` or `NN` after a noun

#### Scenario: A semicolon

- **WHEN** a sentence is `The file holds rows; each row has a key.`
- **THEN** `;` reads `:`, and the model alone reads `,`

#### Scenario: A verb after a noun phrase

- **WHEN** a sentence is `The freshness harness reads stdout.`
- **THEN** `reads` reads `VBZ`, and the model alone reads `NNS`, which gives `NounStack` a false alert ([tag-corrections 6][tag-corrections-6])
- **AND** in `The layout rules of the widgets.`, `rules` stays `NNS`

[tag-corrections-1]: ../../design.md#1-keep-the-model-and-correct-its-tags-afterwards
[tag-corrections-2]: ../../design.md#measures
[tag-corrections-4]: ../../design.md#4-a-title-keeps-the-models-tags
[tag-corrections-5]: ../../design.md#5-a-word-in-capitals-is-a-name-unless-it-is-a-function-word-or-a-unit
[tag-corrections-6]: ../../design.md#6-a-plural-noun-is-a-verb-when-an-object-follows
