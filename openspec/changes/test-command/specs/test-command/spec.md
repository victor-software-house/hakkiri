## ADDED Requirements

### Requirement: Cases come from rule files and test files

`hakkiri test` SHALL run the `tests` of each rule file and each case of each `*.test.yml` file under the paths it is given, or under the current directory when it is given none.  
A directory walk SHALL skip a dot directory other than the one named.  
It SHALL read any other `.yml` file only when it holds `extends` and `tests`, and SHALL fail when it finds no case.

#### Scenario: A rule with cases and a test file

- **WHEN** `styles/Team/Filler.yml` holds two cases and `test/package.test.yml` holds three
- **THEN** `hakkiri test` runs five cases

#### Scenario: YAML that is not a rule

- **WHEN** a directory holds `ci.yml`, a workflow with no `extends`
- **THEN** hakkiri reads no case from it and reports nothing about it

#### Scenario: No case

- **WHEN** the paths hold no case
- **THEN** hakkiri reports an error that no case was found, and exits 2

### Requirement: A case is checked before it runs

A case SHALL have a `name` that is unique in its file, and at least one of `want`, `contains` and `absent`.  
hakkiri SHALL refuse a field a case does not define, with its line.

#### Scenario: A case that asserts nothing

- **WHEN** a case has `name` and `input` and none of the three
- **THEN** hakkiri reports that the case needs one of `want`, `contains` or `absent`, and the case does not run

### Requirement: A case sees alerts as lines

A case's alerts SHALL be one line each, `line:column:Rule:message`, in order of line and column.  
`contains` and `absent` SHALL test the lines for a substring, and `want` SHALL equal them once both are trimmed.

#### Scenario: Contains and want

- **WHEN** `Team.Filler` flags `very` at line 1, column 7, with the message `Remove 'very'.`
- **THEN** `contains: Team.Filler` and `want: "1:7:Team.Filler:Remove 'very'."` both pass

### Requirement: An isolated case loads one rule

A case in a rule's `tests`, and a case with `rule`, SHALL load that rule alone, from a path relative to its file.  
It SHALL read no config, report an alert of any level, and name the rule by its folder and file name.  
Any other case SHALL lint `input` as a file named `input.<format>` through the project config.

#### Scenario: A rule turned off in the config

- **WHEN** a glob sets `Team.Filler: off` and an isolated case for `Team.Filler` has `contains: Team.Filler`
- **THEN** the case passes

#### Scenario: A case through the config

- **WHEN** a case has no `rule` and `files` maps `*.md` to `styles: [Team]`
- **THEN** the case lints its input with the rules of `Team`

### Requirement: Coverage names rules no case fires

With `--coverage`, hakkiri SHALL fail when a rule file under the paths produced no alert in any case, and SHALL name each such rule.  
It SHALL skip dot and underscore folders when it lists the rules.

#### Scenario: A rule no case fires

- **WHEN** `Team.Filler` and `Team.Hedge` each hold a case and only `Team.Filler` flags anything
- **THEN** `hakkiri test --coverage` names `Team.Hedge`, and exits 1

### Requirement: The report and exit status show what happened

hakkiri SHALL print, for each failed case, its file, name and reason, and the alerts it got, and then a count line.  
`--format json` SHALL print `passed`, `failed`, `results` and `uncovered`.  
It SHALL exit 0 when every case passes and no rule is uncovered, 1 when a case fails or a rule is uncovered, and 2 when a case could not run, after every other case has reported.

#### Scenario: A failed case

- **WHEN** a case named `flags filler` has `contains: ProseStyle.Filler` and the rule reports nothing
- **THEN** hakkiri prints its file, `flags filler` and `output does not contain "ProseStyle.Filler"`, and exits 1

#### Scenario: A case that cannot run

- **WHEN** a case names `rule: Missing.yml` and another case fails
- **THEN** hakkiri reports both after running every case, and exits 2

#### Scenario: JSON with colour forced

- **WHEN** `hakkiri test --format json` runs with `CLICOLOR_FORCE=1`
- **THEN** the output parses as JSON and holds no ESC byte
