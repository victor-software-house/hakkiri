## ADDED Requirements

### Requirement: An occurrence counts no code span

hakkiri SHALL count no match of an `occurrence` rule that overlaps a code span, where Vale 3.26.0 reads a code span as a run of `*`, which a token written for it counts.

#### Scenario: A token that reads Vale's mask

- **WHEN** a rule has `extends: occurrence`, `scope: sentence`, `max: 2` and `token: '(?:\b\w+\b|\*+)'`, and the input is ``Run `a`, `b`, `c` and `one two three` today.``
- **THEN** hakkiri counts 3, and Vale counts 7, since each code span is one run of `*` ([code-span-size 1][code-span-size-1])

#### Scenario: A word token

- **WHEN** the rule above has `token: '\b\w+\b'`
- **THEN** both tools count 3

### Requirement: A word count matches Vale with a code span size of 0

hakkiri SHALL count a code span as `code_span_size` words in a `metric` or `readability` rule, where Vale 3.26.0 counts none.  
With `code_span_size` set to 0 the counts, and the alerts of a rule that reads them, SHALL equal Vale's.

#### Scenario: Words of a sentence

- **WHEN** a rule has `extends: metric`, `scope: sentence`, `formula: words` and `condition: '> 2'`, and the input is ``Run `a`, `b`, `c` and `one two three` today.``
- **THEN** hakkiri counts 7 by default and 3 with `code_span_size: 0`, and Vale counts 3 ([code-span-size 3][code-span-size-3])

#### Scenario: A sentence that starts with a code span

- **WHEN** a rule has `extends: metric`, `scope: sentence`, `formula: words` and `condition: '> -1'`, with `code_span_size: 0`, and the input is ``First sentence. `Second` starts with code. Third one.``
- **THEN** hakkiri reports 2.00, 3.00 and 2.00, and Vale reports 2.00 and 2.00, with no alert for the middle sentence, which starts with the code span ([code-span-size 3][code-span-size-3])

[code-span-size-1]: ../../design.md#1-the-engine-keeps-the-counts-of-the-specs
[code-span-size-3]: ../../design.md#3-the-key-is-code_span_size-a-number-of-words
