## ADDED Requirements

### Requirement: An occurrence counts no code span

hakkiri SHALL count no match of an `occurrence` rule that overlaps a code span, where Vale 3.26.0 reads a code span as a run of `*`, which a token written for it counts.

#### Scenario: A token that reads Vale's mask

- **WHEN** a rule has `extends: occurrence`, `scope: sentence`, `max: 2` and `token: '(?:\b\w+\b|\*+)'`, and the input is ``Run `a`, `b`, `c` and `one two three` today.``
- **THEN** hakkiri counts 3, and Vale counts 7, since each code span is one run of `*` ([code-span-words 1][code-span-words-1])

#### Scenario: A word token

- **WHEN** the rule above has `token: '\b\w+\b'`
- **THEN** both tools count 3

[code-span-words-1]: ../../design.md#1-the-engine-keeps-the-counts-of-the-specs
