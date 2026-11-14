# rule-parameters Specification

## Purpose

A glob of the project config sets fields of rules by name, as Vale's `Style.Rule[param] = value` does, and each file runs the rule read with the parameters that apply to it.

## Requirements

### Requirement: A glob sets the parameters of a rule

hakkiri SHALL read `params` in a glob of `files` as a mapping from a rule name to a mapping from a parameter name to one value, a number, `true` or `false`, or text.
It SHALL read the rule again with each parameter replacing the field of that name, whether the rule comes from a rule file or is `Vale.Repetition`.

#### Scenario: A parameter of the built-in rule

- **WHEN** a glob lists `styles: [Vale]` and sets `Vale.Repetition` to `max: 1`, and the text is `the the the`
- **THEN** hakkiri reports the second repeat only, as Vale does for `Vale.Repetition[max] = 1`

#### Scenario: A parameter of a rule file

- **WHEN** a glob sets `ignorecase: true` for an `existence` rule, `grade: 100` for a `readability` rule, or `match: $title` for a `capitalization` rule
- **THEN** hakkiri gives the alerts Vale gives for the same parameter, over every check type that has one

#### Scenario: A value read as the field's kind

- **WHEN** a glob sets `metrics: Gunning Fog` for a `readability` rule
- **THEN** hakkiri reads the text as a list of one metric, as a rule file does

### Requirement: A parameter reaches the files of its glob

A parameter SHALL apply to the files that the glob setting it matches.
The globs that match a file SHALL apply in the order of the settings cascade of `project-config`, a more specific glob winning for the same parameter of the same rule.
An empty `styles` list SHALL drop the parameters of the globs that rank below it.

#### Scenario: A file outside the glob

- **WHEN** `'*.md'` lists `styles: [Vale]` and only `'a.md'` sets `max: 1` for `Vale.Repetition`
- **THEN** `a.md` reports the second repeat of `the the the` and `b.md` reports both

#### Scenario: A more specific glob

- **WHEN** `'a.md'` sets `max: 2` and then `'*.md'` sets `max: 1`
- **THEN** `a.md` reports no repeat of `the the the` and `b.md` reports the second

#### Scenario: An empty styles list

- **WHEN** `'*.md'` lists `styles: [Vale]` and sets `max: 2`, and `'a.md'` has `styles: []` and `Vale.Repetition: on`
- **THEN** `a.md` runs `Vale.Repetition` with `max` 0 and `b.md` with `max` 2

#### Scenario: A package and the project

- **WHEN** a package's settings set `max: 2` for `*.md` and the project sets `max: 1` for `b.md`
- **THEN** `b.md` runs with `max` 1 and `a.md` with `max` 2

### Requirement: A refused parameter names its config line

hakkiri SHALL refuse a parameter with the config file and the line of the parameter, and SHALL report every refusal of a config at once.
A parameter of a rule that no glob runs SHALL be refused in the same way.

#### Scenario: Several faults

- **WHEN** a config sets `max: abc`, `bogus: 1` and `tokens: x` for `Vale.Repetition` and a parameter for `Nope.Rule`
- **THEN** hakkiri refuses all four, each on its own line

#### Scenario: A rule no glob runs

- **WHEN** a glob lists `styles: [Vale]` and sets `max: 1` for `Team.Exist`, an `existence` rule
- **THEN** hakkiri refuses it with the line of `max`

### Requirement: A parameter a rule cannot take is refused

hakkiri SHALL refuse a level, a field that holds a list, a mapping, the rule's identity or its prose, and a field the rule's check does not define.
It SHALL refuse a value of the wrong kind, a missing value, and a list or a mapping as a value.

#### Scenario: A level

- **WHEN** a glob sets `level: warning` as a parameter of `Vale.Repetition`
- **THEN** hakkiri refuses it and says to set the level with the rule's own setting, `Vale.Repetition: warning`

#### Scenario: A field a parameter does not set

- **WHEN** a glob sets `tokens`, `swap`, `exceptions`, `raw`, `either`, `message` or `extends`
- **THEN** hakkiri refuses it and says to extend the rule in a style instead

#### Scenario: A field the check does not define

- **WHEN** a glob sets `max` for an `existence` rule
- **THEN** hakkiri refuses it with the line of `max`

#### Scenario: A value of the wrong kind

- **WHEN** a glob sets `max` to `abc`, `-1` or `1.5`, or `ignorecase` to `maybe`, `YES` or `1`
- **THEN** hakkiri refuses each, with the line and the value it found

### Requirement: A parameter needs a rule that takes one

hakkiri SHALL refuse a parameter of a rule that no loaded style holds, and a parameter of a rule that a vocabulary builds.

#### Scenario: A rule no style holds

- **WHEN** a glob sets `max: 1` for `Nope.Rule`
- **THEN** hakkiri refuses it, as it refuses a level for a rule no style holds

#### Scenario: A rule built from a vocabulary

- **WHEN** a glob sets a parameter of `Vale.Terms`
- **THEN** hakkiri refuses it, since the vocabulary builds the rule and it has no field to set
