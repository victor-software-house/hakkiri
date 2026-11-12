# test-command Specification

## Purpose

`hakkiri test` runs the cases that rule files and `*.test.yml` files hold, checks each against the alerts it gets, and reports the result and an exit status.

## Requirements

### Requirement: Cases come from rule files and test files

`hakkiri test` SHALL run the `tests` of each rule file and each case of each `*.test.yml` file under the paths it is given, or under the current directory when it is given none.  
A directory walk SHALL skip a dot directory other than the one named, and every folder the `check` walk skips by name.  
It SHALL read any other `.yml` file only when it holds `extends` and `tests`, and SHALL fail when it finds no case.

#### Scenario: A rule with cases and a test file

- **WHEN** `styles/Team/Filler.yml` holds two cases and `test/package.test.yml` holds three
- **THEN** `hakkiri test` runs five cases

#### Scenario: YAML that is not a rule

- **WHEN** a directory holds `ci.yml`, a workflow with no `extends`
- **THEN** hakkiri reads no case from it and reports nothing about it

#### Scenario: A folder the walk skips

- **WHEN** `node_modules/pkg/x.test.yml` holds a failing case and the rest of the project holds passing ones
- **THEN** `hakkiri test` runs no case from `node_modules` and exits 0

#### Scenario: A folder named as an argument

- **WHEN** `hakkiri test node_modules/pkg` runs and `pkg` holds a case, and `pkg/node_modules` holds a failing one
- **THEN** hakkiri runs the case in `pkg` and none from `pkg/node_modules`, as `check` does

#### Scenario: A skipped name given as the folder

- **WHEN** `hakkiri test node_modules` runs and `node_modules` holds a case
- **THEN** hakkiri runs the case, because a folder named as an argument is walked whatever its name

#### Scenario: No case

- **WHEN** the paths hold no case
- **THEN** hakkiri reports an error that no case was found, and exits 2

### Requirement: A rule file may hold cases

A rule file SHALL take `tests` as the cases of [design decision 2][test-2].  
`hakkiri check` SHALL load them and run none, and SHALL refuse a malformed `tests` with its line.

#### Scenario: A rule with cases under check

- **WHEN** `hakkiri check` runs over a style whose rule holds a case that would fail
- **THEN** hakkiri reports no failure for the case, and exits by the alerts alone

#### Scenario: A malformed case in a rule

- **WHEN** a rule's `tests` holds a case with the field `inputs`
- **THEN** hakkiri refuses the rule, names the line of `inputs`, and exits 2

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
It SHALL read no config, report an alert of any level, and name the rule as the style loader does, so `Team/Foo.v2.yml` is `Team.Foo`.  
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

#### Scenario: A rule file with a dotted name

- **WHEN** `styles/Team/Foo.v2.yml` flags a line that a case through the config lints, under `--coverage`
- **THEN** hakkiri counts `Team.Foo` as covered, the name the alert carries

### Requirement: The report and exit status show what happened

hakkiri SHALL print, for each failed case, its file, name and reason, and the alerts it got, and then a count line.  
`--format json` SHALL print `passed`, `failed`, `files`, `results`, `uncovered` and `failures`.  
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

[test-2]: ../../changes/archive/2026-11-10-test-command/design.md#2-a-case-is-a-small-record

### Requirement: The cases of a rule hakkiri cannot run are not run

`hakkiri test` SHALL report the cases of a rule hakkiri cannot run as not run, with a failure that says its cases did not run.  
It SHALL report a case that runs through a config as not run when the config runs such a rule on the case's file.

#### Scenario: A script rule with a case

- **WHEN** `Cannot.yml` holds `extends: script` and one case, and `Parent.yml` holds a case that passes
- **THEN** hakkiri prints a failure that says its cases did not run, counts the passing case, and exits 2

#### Scenario: A case through a config that runs a rule hakkiri cannot run

- **WHEN** a test file holds a case that runs through a config whose listed style holds a script rule
- **THEN** hakkiri reports the case as not run, naming the rule, and exits 2

### Requirement: A case's format picks its reader

A case that runs through the config SHALL read its input by the reader a `formats` entry gives `input.<format>`, and otherwise by the extension `<format>`.  
An isolated case SHALL read its input by the extension `<format>`.  
A case whose format no reader reads SHALL read its input as plain text, except that a case through the config follows `on_unknown`.

#### Scenario: A format no extension names

- **WHEN** a case has `format: txt` and `input: This is very good.`, and a rule flags `very`
- **THEN** the case lints the input as plain text and reports `1:9`, both in the rule's `tests` and in a test file

#### Scenario: A formats entry

- **WHEN** the config maps `cfg: md`, a glob selects `*.cfg`, and a case has `format: cfg` and a fenced block that holds `very`
- **THEN** the case reads the input as Markdown and the rule reports nothing for the fence

### Requirement: A case through the config has the extensions of its file

A case that runs through the config SHALL have the extensions that `check` gives the file `input.<format>`.
Those are `<format>` and the extension of the reader, which a `formats` entry may give.

#### Scenario: A mapped case

- **WHEN** the config maps `cfg: md`, a rule has `scope: text.md`, and a case has `format: cfg` and an input that the rule flags
- **THEN** the case reports the alert

### Requirement: A case through the config follows on_unknown

A case through the config whose format no reader reads, and whose file the config selects, SHALL follow `on_unknown` as `check` does.  
`text` reads it as plain text, `warn` and `ignore` give it no alerts, and `error` makes the case fail to run with a message that names the format.

#### Scenario: An error

- **WHEN** the config holds `on_unknown: error` and `'*.txt': {styles: [Team]}`, and a case has `format: txt`
- **THEN** the case fails to run with ``no reader reads the format `txt` ``, and the run exits 2

#### Scenario: A warning or an ignore

- **WHEN** the config holds `on_unknown: warn`, or `ignore`, and a case has `format: txt` with `want: ''` and an input that a rule flags in plain text
- **THEN** the case passes, because the config does not lint such a file
