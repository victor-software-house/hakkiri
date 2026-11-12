## REMOVED Requirements

### Requirement: The built-in repetition rule is a known gap

**Reason**: The style `Vale` now holds `Vale.Repetition`, so the gap closes.

**Migration**: Read the requirement "Vocabulary rules differ from Vale's in this way" under the ADDED section for what still differs.

### Requirement: Vocabulary rules differ from Vale's in these ways

**Reason**: The style `Vale` now exists with no term, so the scenario for it closes, and a scenario cannot be dropped alone.

**Migration**: Read the requirement under the ADDED section, named "Vocabulary rules differ from Vale's in this way".

## ADDED Requirements

### Requirement: Vocabulary rules differ from Vale's in this way

hakkiri SHALL build and run the rules of a vocabulary as Vale does, apart from the scenario below.

#### Scenario: An escape in a term

- **WHEN** `data\Wset` is accepted and the text is `A data set, data-set, dataxset and DATA SET.`
- **THEN** hakkiri flags `DATA SET`, since it lowercases the literal text of the term and keeps `\W`, and Vale flags `dataxset`, since it lowercases the escape to `\w` ([vocabulary-rules 1][vocabulary-rules-1])

[vocabulary-rules-1]: ../../../archive/2026-11-11-vocabulary-rules/design.md#1-terms-and-avoid-are-rules-of-a-style-named-vale
