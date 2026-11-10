# Spec Delta

## MODIFIED Requirements

### Requirement: A metric rule counts token matches

A `metric` rule with `tokens` SHALL give its formula the variable `occurrences`, the number of matches of those tokens in the text it measures.

#### Scenario: A semicolon rate

- **WHEN** a rule has `extends: metric`, `message: "%s semicolons per 1,000 words, over 3: split the sentences."`, `nonword: true`, `tokens: [';']`, `formula: occurrences / words * 1000` and `condition: '> 3'`, and the input is `One; two; three four five six seven eight nine ten.`
- **THEN** hakkiri reports one alert with the message `200.00 semicolons per 1,000 words, over 3: split the sentences.`

## ADDED Requirements

### Requirement: Metric tokens match as existence tokens do

The `tokens` of a `metric` rule SHALL match as the `tokens` of an `existence` rule do, with word boundaries unless `nonword` is `true`, and with `ignorecase`, `raw` and `exceptions`.

#### Scenario: A semicolon with word boundaries

- **WHEN** the semicolon rule above has no `nonword` and reads `One; two; three four five six seven eight nine ten.`
- **THEN** its formula reads `occurrences` as 0, and hakkiri reports no alert
