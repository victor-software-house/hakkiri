## ADDED Requirements

### Requirement: Pattern checks differ from Vale

hakkiri SHALL run the pattern checks as each scenario below says, where Vale 3.26.0 runs them otherwise.

#### Scenario: State belongs to one rule

- **WHEN** one `conditional` rule captures a term, and another rule of either kind reads the same file
- **THEN** hakkiri keeps each rule's state apart, and Vale keeps one list per file, so a captured term can satisfy another `conditional` rule or count as a `consistency` form ([pattern 1][pattern-1])

#### Scenario: A consistency alert sits at the last match

- **WHEN** a `consistency` pair matches in a node whose earlier text holds the same words
- **THEN** hakkiri puts the alert at the last match through its byte table, and Vale searches for the text, so the alert lands where it first appears ([pattern 2][pattern-2])

#### Scenario: ignorecase applies to both conditional patterns

- **WHEN** a `conditional` rule has `ignorecase: true`
- **THEN** hakkiri matches `first` and `second` without regard to case, and Vale decodes the field and reads it nowhere ([pattern 7][pattern-7])

#### Scenario: Code is no match

- **WHEN** two code spans sit side by side, or a form appears only in code
- **THEN** hakkiri counts no repeat and no form seen, and Vale masks the code, so it can report a repeat the reader does not see ([pattern 9][pattern-9])

### Requirement: Rules that Vale loads are refused

hakkiri SHALL refuse at load each rule below, which Vale 3.26.0 loads.

#### Scenario: A conditional with an empty pattern

- **WHEN** a `conditional` rule has a `first` or `second` that is empty or matches the empty string
- **THEN** hakkiri refuses it, and Vale compiles both as written ([pattern 3][pattern-3])

#### Scenario: An unknown field in a consistency rule

- **WHEN** a `consistency` rule sets a field it does not define
- **THEN** hakkiri refuses it, and Vale decodes this check loosely and accepts it ([pattern 2][pattern-2])

#### Scenario: A readability metrics list that is empty or unknown

- **WHEN** a `readability` rule has an empty `metrics` list, or a name outside the five scores
- **THEN** hakkiri refuses it, and Vale adds 0 for the unknown name and still counts it in the divisor ([pattern 6][pattern-6])

#### Scenario: A capitalization match that is empty

- **WHEN** a `capitalization` rule has an empty `match`
- **THEN** hakkiri refuses it, since every text would pass, and Vale loads it ([case 1][case-1])

#### Scenario: A capitalization style that is not AP or Chicago

- **WHEN** a `capitalization` rule has `style: chicago` or any other value than `AP` or `Chicago`
- **THEN** hakkiri refuses it with an error that names the value, and Vale reads every other value as `AP` ([case 8][case-8])

#### Scenario: A sequence with nothing to start from

- **WHEN** no token of a `sequence` rule has a `pattern`, `tag` or `upos` without `negate`
- **THEN** hakkiri refuses the rule, and Vale loads it and never matches ([case 3][case-3])

#### Scenario: A sequence token with a field or count it cannot use

- **WHEN** a `sequence` token sets a field it does not define, or a `skip` or `min` below 0, or one whose steps overflow a count
- **THEN** hakkiri refuses the rule, and Vale loads it ([case 3][case-3])

#### Scenario: A sequence model

- **WHEN** a `sequence` rule sets `model`
- **THEN** hakkiri refuses it, and Vale reads another checker's tagger model ([case 4][case-4])

#### Scenario: A spelling rule with exceptions or threshold

- **WHEN** a `spelling` rule sets `exceptions`, or a `threshold` other than 0
- **THEN** hakkiri refuses it with a message that the field has no effect, and Vale loads it and reads neither ([spelling 3][spelling-3])

#### Scenario: A dictionary outside the package

- **WHEN** a rule names a dictionary, list or affix file by a path that leaves its package, or sets only one of `aff` and `dic`
- **THEN** hakkiri refuses it, and Vale also searches an absolute path, the working directory and `DICPATH`, and answers one of `aff` and `dic` with the built-in dictionary ([spelling 2][spelling-2])

#### Scenario: A dictionary in another character set

- **WHEN** a dictionary declares a `SET` other than UTF-8 and ISO 8859-1
- **THEN** hakkiri refuses it with an error that names the set, and Vale reads every set that `golang.org/x/text` knows ([case 6][case-6])

[case-1]: ../../../archive/2026-11-10-case-and-sequence-checks/design.md#1-case-conversion-ports-proses-converters
[case-3]: ../../../archive/2026-11-10-case-and-sequence-checks/design.md#3-sequence-walks-tokens-around-an-anchor
[case-4]: ../../../archive/2026-11-10-case-and-sequence-checks/design.md#4-model-is-refused
[case-6]: ../../../archive/2026-11-10-case-and-sequence-checks/design.md#6-spelling-ports-vales-hunspell-reader
[case-8]: ../../../archive/2026-11-10-case-and-sequence-checks/design.md#8-a-style-other-than-ap-or-chicago-is-refused
[pattern-1]: ../../../archive/2026-11-10-pattern-and-readability-checks/design.md#1-per-file-state-belongs-to-one-rule
[pattern-2]: ../../../archive/2026-11-10-pattern-and-readability-checks/design.md#2-consistency-matches-as-existence-does-one-pattern-per-pair
[pattern-3]: ../../../archive/2026-11-10-pattern-and-readability-checks/design.md#3-conditional-has-two-modes-chosen-by-the-second-pattern
[pattern-6]: ../../../archive/2026-11-10-pattern-and-readability-checks/design.md#6-an-unknown-or-empty-metrics-list-is-refused
[pattern-7]: ../../../archive/2026-11-10-pattern-and-readability-checks/design.md#7-ignorecase-on-a-conditional-applies-to-both-patterns
[pattern-9]: ../../../archive/2026-11-10-pattern-and-readability-checks/design.md#9-code-is-no-match-in-the-pattern-checks
[spelling-2]: ../../../archive/2026-11-10-spelling-and-affix-checks/design.md#2-dictionaries-and-lists-stay-inside-the-package
[spelling-3]: ../../../archive/2026-11-10-spelling-and-affix-checks/design.md#3-fields-vale-reads-and-never-uses
