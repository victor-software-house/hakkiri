# Spec Delta

## Purpose

Loading turns the rule files of a style folder into rules the engine can run.  
It refuses a rule it cannot run, with an error that names the file, the field and the line.

## ADDED Requirements

### Requirement: A style folder gives one rule per file

hakkiri SHALL load every `.yml` file directly inside a style folder as one rule, and SHALL ignore every other file there.

#### Scenario: A folder with a rule and a README

- **WHEN** a style folder `Team` holds `Filler.yml` and `README.md`
- **THEN** hakkiri loads one rule, `Team.Filler`

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
