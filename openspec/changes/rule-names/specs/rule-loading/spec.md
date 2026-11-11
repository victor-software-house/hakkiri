## MODIFIED Requirements

### Requirement: A style folder gives one rule per file

hakkiri SHALL load every `.yml` file under a style folder as one rule, in a folder that does not start with `.` or `_`, and SHALL ignore every other file there.  
A file named `*.test.yml` holds cases and is no rule.

#### Scenario: A folder with a rule and a README

- **WHEN** a style folder `Team` holds `Filler.yml` and `README.md`
- **THEN** hakkiri loads one rule, `Team.Filler`

#### Scenario: Files that are no rules

- **WHEN** the style folder `Team` holds `Filler.test.yml`, `_draft/Old.yml` and `.hidden/Old.yml`, none with an `extends`
- **THEN** hakkiri loads no rule from them and reports no error

## ADDED Requirements

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
