# rule-loading Specification

## Purpose

Loading turns the rule files of a style folder into rules the engine can run.  
It refuses a rule it cannot run, with an error that names the file, the field and the line.

## Requirements

### Requirement: A style folder gives one rule per file

hakkiri SHALL load every `.yml` file under a style folder as one rule, in a folder that does not start with `.` or `_`, and SHALL ignore every other file there.  
A file named `*.test.yml` holds cases and is no rule.

#### Scenario: A folder with a rule and a README

- **WHEN** a style folder `Team` holds `Filler.yml` and `README.md`
- **THEN** hakkiri loads one rule, `Team.Filler`

#### Scenario: Files that are no rules

- **WHEN** the style folder `Team` holds `Filler.test.yml`, `_draft/Old.yml` and `.hidden/Old.yml`, none with an `extends`
- **THEN** hakkiri loads no rule from them and reports no error

### Requirement: A rule needs extends and message

hakkiri SHALL refuse a rule file that has no `extends` or no `message`, and report an error that names the file and the missing field.

#### Scenario: A rule with no message

- **WHEN** `Team/Short.yml` holds `extends: existence` and `tokens: [very]` only
- **THEN** hakkiri reports an error naming `Team/Short.yml` and `message`, and exits with a failure status

### Requirement: An unknown extends is refused

hakkiri SHALL refuse a rule whose `extends` names no check type, and report an error that names the file and the value.

#### Scenario: A check type Vale does not define

- **WHEN** `Team/Grammar.yml` holds `extends: grammar`
- **THEN** hakkiri reports an error naming `Team/Grammar.yml` and `grammar`, and exits with a failure status

### Requirement: An unknown field is refused

hakkiri SHALL refuse a rule that holds a field its check type does not define.  
The error SHALL name the field and its line.

#### Scenario: token in an existence rule

- **WHEN** an `existence` rule holds `token: very` on line 3, where the type defines `tokens`
- **THEN** hakkiri reports an error naming `token` and line 3, and exits with a failure status

### Requirement: A field of the wrong type is refused

hakkiri SHALL refuse a rule whose field holds a value of the wrong type, and report an error that names the field and its line.  
A number written as a string, such as `max: '3'`, SHALL read as the number.

#### Scenario: A count that is not a number

- **WHEN** an `occurrence` rule holds `max: many` on line 4
- **THEN** hakkiri reports an error naming `max` and line 4

#### Scenario: A number in quotes

- **WHEN** an `occurrence` rule holds `max: '3'`
- **THEN** hakkiri loads the rule with a `max` of 3

### Requirement: level takes three values

A rule's `level` SHALL be `suggestion`, `warning` or `error`, and SHALL be `warning` when the rule sets none.  
hakkiri SHALL refuse any other value and name the file and the field.

#### Scenario: A rule with no level

- **WHEN** an `existence` rule with `tokens: [very]` and no `level` reads `It is very good.`
- **THEN** hakkiri reports one warning

#### Scenario: A level Vale does not define

- **WHEN** a rule holds `level: info`
- **THEN** hakkiri reports an error naming the file and `level`, and exits with a failure status

### Requirement: A pattern that does not compile is refused

hakkiri SHALL refuse a rule with a pattern that neither regex engine compiles, and report an error that names the rule and the pattern.

#### Scenario: An unclosed group

- **WHEN** `Team.Broken` has `extends: existence` and `tokens: ['(unclosed']`
- **THEN** hakkiri reports an error naming `Team.Broken` and `(unclosed`, and exits with a failure status

### Requirement: A pattern with nothing to match is refused

hakkiri SHALL refuse an `existence` rule whose `tokens` and `raw` hold nothing to match, and a `metric` rule that lists `tokens` or `raw` holding nothing, because such a pattern matches at every position.  
The error SHALL name the file, the line, `tokens` and `raw`.

#### Scenario: An existence rule with no tokens

- **WHEN** `Team/Empty.yml` holds `extends: existence` and `message: m` only
- **THEN** hakkiri reports an error naming `Team/Empty.yml`, line 1, `tokens` and `raw`, and exits with a failure status

#### Scenario: Tokens of blanks only

- **WHEN** an `existence` rule holds `tokens: ['', '  ']` and no `raw`
- **THEN** hakkiri reports the same error

#### Scenario: A metric with an empty raw

- **WHEN** a `metric` rule holds `raw: ['']` and no `tokens`
- **THEN** hakkiri reports the same error

### Requirement: A YAML error names the line

hakkiri SHALL refuse a rule file that is not valid YAML, and report an error that names the file and the line of the fault.

#### Scenario: A tab in the indentation

- **WHEN** `Team/Tabs.yml` holds a tab character in the indentation of line 5
- **THEN** hakkiri reports an error naming `Team/Tabs.yml` and line 5

### Requirement: An action loads and is not applied

hakkiri SHALL load a rule's `action` field, carry it on the rule's alerts, and change no text with it.

#### Scenario: An existence rule with an action

- **WHEN** a rule has `extends: existence`, `message: "Remove '%s'."`, `tokens: [very]` and `action: {name: remove}`, and the input is `It is very good.`
- **THEN** hakkiri reports one alert with the message `Remove 'very'.`, and the file is unchanged

### Requirement: A rule is named by its path

hakkiri SHALL name a rule by its style folder, each folder between it and the file, and the file name up to its first dot, joined by dots, as Vale does.  
A name with `[` or `]` SHALL be refused when a glob lists the style.  
`check`, `test` and `test --coverage` SHALL give a file one name.

#### Scenario: A dotted file name

- **WHEN** the style folder `Team` holds `Terms.custom.yml` and `Foo.v2.yml`
- **THEN** the rules are `Team.Terms` and `Team.Foo`, and a config key `Team.Terms: off` turns the first one off

#### Scenario: A nested rule

- **WHEN** the style folder `Team` holds `Sub/Bar.yml` and `Sub/Deep/Baz.yml`
- **THEN** the rules are `Team.Sub.Bar` and `Team.Sub.Deep.Baz`, in `check` and in the `test` cases that name them

#### Scenario: A nested rule under coverage

- **WHEN** the config has the package `styles`, `styles/Team/Sub/Deep/Bar.yml` flags `perhaps`, and a case through the config expects `Team.Sub.Deep.Bar`
- **THEN** `test --coverage` passes the case and lists no uncovered rule

#### Scenario: A bracket in a name

- **WHEN** the style folder `Team` holds `Weird[max].yml`, and a glob lists `Team`
- **THEN** hakkiri refuses it with an error that names the file and the rule name `Team.Weird[max]`

### Requirement: Two files with one name are refused

hakkiri SHALL refuse a style folder in which two rule files give one rule name, when a glob lists the style, with an error that names the rule and both files.  
A style no glob lists SHALL load with no error, as in Vale.

#### Scenario: Terms.yml and Terms.custom.yml

- **WHEN** the style folder `Team` holds `Terms.yml` and `Terms.custom.yml`, and a glob lists `Team`
- **THEN** hakkiri exits with a failure status, and the error names `Team.Terms`, `Terms.yml` and `Terms.custom.yml`

#### Scenario: A clash or a bracket in a style no glob lists

- **WHEN** the style folder `Team` holds `Terms.yml` and `Terms.custom.yml`, or `Weird[max].yml`, and no glob lists `Team`
- **THEN** hakkiri lints with the other styles and exits as it would without `Team`
