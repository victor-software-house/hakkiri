# Spec Delta

## Purpose

An alert is what a rule reports: which rule, at what level, with what message, and where in the file.  
This spec fixes its fields, its position, its message, its order, and what a pattern failure reports.

## ADDED Requirements

### Requirement: An alert names the rule, level, message, place and match

Each alert SHALL hold the rule name, the level, the message, the line, the first and last column of the match, and the matched text.  
It SHALL also hold the rule's `link` and `description` when the rule sets them.

#### Scenario: A word in a sentence

- **WHEN** a rule `Team.Very` has `extends: existence`, `message: "Drop '%s'."`, `level: error` and `tokens: [very]`, and the input is `It is very good.`
- **THEN** hakkiri reports one alert for `Team.Very`, level `error`, message `Drop 'very'.`, line 1, columns 7 to 10, matched text `very`

### Requirement: Columns count characters

Lines and columns SHALL start at 1, and a column SHALL count characters, not bytes, as Vale 3.26.0 does.

#### Scenario: A letter with an accent before the match

- **WHEN** the rule above reads `Café very good.`
- **THEN** the alert is on line 1, columns 6 to 9

### Requirement: A position inside a comment points into the file

The line and columns of a match inside a comment SHALL be those of its bytes in the file.

#### Scenario: An indented doc comment

- **WHEN** a rule has `scope: text.comment` and `tokens: [marker]`, and a Rust file holds a line of four spaces and `/// A marker.`
- **THEN** the alert is on that line, columns 11 to 16

### Requirement: A message fills %s in order

Each `%s` in a message SHALL take the next value the check gives, and `%%` SHALL read as `%`.  
A line break inside a value SHALL read as one space.

#### Scenario: A percent sign

- **WHEN** a rule has `message: "'%s' is 100%% filler."`, `ignorecase: true` and `tokens: [basically]`, and the input is `Basically, yes.`
- **THEN** the alert message is `'Basically' is 100% filler.`

#### Scenario: A match across a line break

- **WHEN** a rule has `message: "'%s'"` and `tokens: ['very good']`, and the input is `It is very` and `good.` on two lines
- **THEN** the alert message is `'very good'`

### Requirement: Alerts come in file order

hakkiri SHALL return the alerts of a file sorted by line, then by column.  
Alerts at the same place SHALL keep the order of their rules.

#### Scenario: Two rules on one line

- **WHEN** rule `Team.B` flags `good` and rule `Team.A` flags `very`, and the input is `It is very good.`
- **THEN** the `Team.A` alert at column 7 comes before the `Team.B` alert at column 12

### Requirement: One alert per rule and place

hakkiri SHALL drop an alert whose rule, line and first column equal those of an earlier alert, as Vale 3.26.0 does.

#### Scenario: A list item inside a quote

- **WHEN** a rule has `scope: [list, blockquote]` and `tokens: [marker]`, and the input is `> - A marker`
- **THEN** hakkiri reports one alert

### Requirement: limit caps a rule's alerts

A rule with a `limit` above 0 SHALL report at most that many alerts in one file, the first ones in file order.

#### Scenario: A limit of one

- **WHEN** a rule has `ignorecase: true`, `tokens: [very]` and `limit: 1`, and the input is `Very good, very fast.`
- **THEN** hakkiri reports one alert, at column 1

### Requirement: A pattern at the backtrack limit is an error alert

When a pattern reaches the backtrack limit, hakkiri SHALL report one `error` alert for that rule and file, with the limit in its message.  
hakkiri SHALL still run the other rules.  
It SHALL NOT panic.

#### Scenario: Nested repeats with a lookahead

- **WHEN** rule `Team.Slow` has `nonword: true` and `tokens: ['(a+)+(?=b)c']`, and the input is one line of 50,000 `a` characters
- **THEN** hakkiri reports one `error` alert for `Team.Slow` that names the backtrack limit, and the other rules report as usual

### Requirement: Long input finishes

hakkiri SHALL lint a 50,000-character line and a 60,000-line file to the end, with no panic and every alert in place.

#### Scenario: A match at the end of a long line

- **WHEN** a rule has `tokens: [marker]`, and the input is one line of 24,997 pairs of the letter `a` and a space, followed by `marker`
- **THEN** hakkiri reports one alert at line 1, column 49,995

#### Scenario: Sixty thousand lines

- **WHEN** the same rule reads a file of 60,000 lines that each hold `A marker.`
- **THEN** hakkiri reports 60,000 alerts, the last on line 60,000
