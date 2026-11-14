## MODIFIED Requirements

### Requirement: Where an alert points and what it matches differs from Vale

hakkiri SHALL place an alert, and match text for `existence`, `substitution` and `occurrence`, as each scenario below says, where Vale 3.26.0 does otherwise.

#### Scenario: An alert is placed by offset

- **WHEN** a phrase appears twice in a block and the second copy matches
- **THEN** hakkiri points at the second copy through its byte table, and Vale's default path searches for the matched text, so it can land on the first ([engine 7][engine-7])

#### Scenario: A measurement alert points at its node

- **WHEN** a `metric` rule with `scope: paragraph`, `formula: words` and `condition: '> 3'` reads the paragraph `One two three`, `four five six`, `seven eight.` on lines 1 to 3
- **THEN** hakkiri gives line 1, column 1, the first byte of the paragraph, and Vale gives line 3, column 1, the start of the block's last line ([engine 16][engine-16])

#### Scenario: A metric alert on a nested list item

- **WHEN** the same rule with `scope: list` reads the items `Item one two` / `three four five.`, `Short.` with the nested item `Nested one two` / `three four five.`, and `Last item one` / `two three four.`, from line 5
- **THEN** both count the nested item as its own block, so `Short.` gives no alert
- **AND** hakkiri gives lines 5, 8 and 10 past each bullet, at columns 3, 5 and 3, where Vale gives lines 6, 9 and 11 at column 1 ([engine 16][engine-16])

#### Scenario: Word and space classes are Unicode

- **WHEN** a pattern uses `\w`, `\d` or `\s` on `naïve`
- **THEN** hakkiri reads one word, since these classes are Unicode as `\b` is, and Vale reads them as ASCII, so a word with an accent reads as two ([engine 16][engine-16])

#### Scenario: capitalize needs a capital to start

- **WHEN** a swap value is capitalized for a match that starts with a digit or a symbol, such as `1st`
- **THEN** hakkiri capitalizes only a match that starts with a capital letter, and Vale capitalizes the value, so `1st` swaps to `First` ([engine 16][engine-16])

#### Scenario: A swap key with a space reads a line wrap

- **WHEN** a `substitution` key holds a space and `$1` in its value, and the matched text wraps a line
- **THEN** hakkiri expands `$1` from the key as matched, and Vale expands it from the key as written, so the alert is lost ([engine 16][engine-16])

#### Scenario: An occurrence over blank text points at a word

- **WHEN** an `occurrence` rule matches blank text only
- **THEN** hakkiri points at the first word of the node, and Vale counts the matches, points at none and gives no alert ([engine 16][engine-16])

#### Scenario: An occurrence over code is not counted

- **WHEN** an `occurrence` match overlaps a code span
- **THEN** hakkiri does not count it, and Vale counts every match, then points at the first one that is not code ([engine 16][engine-16])

#### Scenario: Overlapping phrase exceptions both hold

- **WHEN** the text is `a b c`, the exceptions are the phrases `a b` and `b c`, and a rule matches `c`
- **THEN** hakkiri holds back the match, and Vale reports `c`, since its one alternation never finds `b c` after `a b` ([engine 18][engine-18])

[engine-16]: ../../../2026-11-10-rule-engine/design.md#16-better-than-vale-where-vale-is-wrong
[engine-18]: ../../../2026-11-10-rule-engine/design.md#18-every-phrase-exception-holds-overlapping-or-nested
[engine-7]: ../../../2026-11-10-rule-engine/design.md#7-positions-from-one-line-table
