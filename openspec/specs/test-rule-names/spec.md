# test-rule-names Specification

## Purpose

How `hakkiri test` gives a rule its name and reads its text from the path it was given.  
So a coverage report and an alert use the same name.

## Requirements

### Requirement: A rule takes its name from the path it was found at

hakkiri SHALL derive a rule's name from its absolute path, with `.` and `..` parts folded away and symlinks left alone.  
The style loader derives the name the same way from the folder it was given.

It SHALL read the rule's text from that same path.  
So a path through a symlink and back out with `..` reads the file its folded form points to.  
A rule file given as `Foo.yml` or `./Foo.yml`, from inside its style folder, SHALL get the name of that folder.

#### Scenario: Run from inside the style folder

- **WHEN** `hakkiri test Foo.yml`, `hakkiri test ./Foo.yml` and `hakkiri test --coverage .` run in `styles/Team`
- **THEN** each runs the cases of `Foo.yml` and calls the rule `Team.Foo`

#### Scenario: Through a symlink and back out

- **WHEN** a case has `rule: via/../Team/Foo.yml` and `via` is a symlink to a folder elsewhere
- **THEN** hakkiri reads `Team/Foo.yml` beside the case file, as Vale does, and not the file the symlink's target contains

#### Scenario: A symlinked style folder

- **WHEN** `styles/Team` is a symlink to `shared/TeamStyle` that contains `Foo.yml`, and a case through the config flags `Team.Foo`
- **THEN** `hakkiri test --coverage styles/Team/Foo.yml` counts `Team.Foo` as covered

#### Scenario: A `.yaml` rule file

- **WHEN** `styles/Team/Other.yaml` contains `extends` and `tests`
- **THEN** `hakkiri test --coverage` reads no case from it and reports no rule for it

#### Scenario: A nested rule

- **WHEN** the config has the package `styles` and a case loads `styles/Team/Sub/Bar.yml`
- **THEN** the rule is `Team.Sub.Bar`, and with no package that contains the file it is `Sub.Bar`, since the folder that contains the file is its style

#### Scenario: A dotted file name

- **WHEN** a case loads `styles/Team/Foo.v2.yml`
- **THEN** the rule is `Team.Foo`

#### Scenario: A project reached through a link

- **WHEN** the project root is reached through a symlink, a style folder is a symlink, and `test --coverage` runs on `link/styles/Team/Sub/Bar.yml`
- **THEN** the rule is `Team.Sub.Bar`, as `check` calls it, and not `Sub.Bar`

#### Scenario: A name two rule files share

- **WHEN** `Terms.yml` and `Terms.custom.yml` both give `Team.Terms`, and a case flags it through either file
- **THEN** `test --coverage` counts the name as covered, as `vale test --coverage` does, and when no case flags it, lists it once with both files

#### Scenario: A shared name in JSON

- **WHEN** `test --coverage --format json` finds that no case flags `Team.Terms`, which `Terms.yml` and `Terms.custom.yml` both give
- **THEN** `uncovered` contains one object, `{"rule": "Team.Terms", "files": [...]}` with both files, and the object of a name one file gives contains `rule` alone
