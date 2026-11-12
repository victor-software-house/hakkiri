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
With `code_span_size` set to 0 the counts, and the alerts of a rule that reads them, SHALL equal Vale's, except where another requirement of this spec records a difference.

#### Scenario: Words of a sentence

- **WHEN** a rule has `extends: metric`, `scope: sentence`, `formula: words` and `condition: '> 2'`, and the input is ``Run `a`, `b`, `c` and `one two three` today.``
- **THEN** hakkiri counts 7 by default and 3 with `code_span_size: 0`, and Vale counts 3 ([code-span-size 3][code-span-size-3])

### Requirement: An alert keeps its own place

hakkiri SHALL report each alert of a `metric` or `readability` rule at the place of its block, where Vale 3.26.0 drops an alert that has the line, the column and the rule of an earlier one.

#### Scenario: A table row

- **WHEN** a rule has `extends: metric`, `scope: table.cell`, `formula: words` and `condition: '> -1'`, and the input is the table `| Head | Other |`, `|:--|:--|`, `| one | two words |`
- **THEN** hakkiri reports 1.00 and 2.00, and Vale reports 1.00 only, since each cell of a row has the first column of its line ([code-span-size 6][code-span-size-6])

#### Scenario: A sentence that starts with a code span

- **WHEN** a rule has `extends: metric`, `scope: sentence`, `formula: words` and `condition: '> -1'`, with `code_span_size: 0`, and the input is ``First sentence. `Second` starts with code. Third one.``
- **THEN** hakkiri reports 2.00, 3.00 and 2.00, and Vale reports 2.00 and 2.00, with no alert for the middle sentence, which starts with the code span and has the place of the first ([code-span-size 6][code-span-size-6])

#### Scenario: A reference definition that repeats a code span

- **WHEN** a rule has `extends: metric`, `scope: paragraph`, `formula: words` and `condition: '> -1'`, with `code_span_size: 0`, and the input is ``See [`a/b`][r].``, a blank line, `Two words here.`, a blank line, `Three words go here.`, a blank line and `[r]: a/b`
- **THEN** hakkiri reports 1.00, 3.00 and 4.00, and Vale reports 1.00 on the line of the definition, with no alert for the other two paragraphs ([code-span-size 6][code-span-size-6])

#### Scenario: An entity in a code span

- **WHEN** the rule above has the input ``First a paragraph holds `&amp;lt; marker` ``, a blank line and ``Second paragraph here reads `&lt; marker` ``
- **THEN** hakkiri reports 4.00 and 4.00, and Vale reports 4.00 once, on the line of the second paragraph ([code-span-size 6][code-span-size-6])

[code-span-size-1]: ../../design.md#1-the-engine-keeps-the-counts-of-the-specs
[code-span-size-3]: ../../design.md#3-the-key-is-code_span_size-a-number-of-words
[code-span-size-6]: ../../design.md#6-vale-drops-an-alert-that-has-the-place-of-another
