## ADDED Requirements

### Requirement: A rule is named from the path it was found at

hakkiri SHALL name a rule from its path made absolute, with `.` and `..` parts folded away and symlinks left alone, as the style loader names it from the folder it was given.  
It SHALL read the rule's text from that same path, so a path through a symlink and back out with `..` reads the file its folded form names.  
A rule file named as `Foo.yml` or `./Foo.yml`, from inside its style folder, SHALL get the name of that folder.

#### Scenario: Run from inside the style folder

- **WHEN** `hakkiri test Foo.yml`, `hakkiri test ./Foo.yml` and `hakkiri test --coverage .` run in `styles/Team`
- **THEN** each runs the cases of `Foo.yml` and names the rule `Team.Foo`

#### Scenario: Through a symlink and back out

- **WHEN** a case has `rule: via/../Team/Foo.yml` and `via` is a symlink to a folder elsewhere
- **THEN** hakkiri reads `Team/Foo.yml` beside the case file, as Vale does, and not the file the symlink's target holds

#### Scenario: A symlinked style folder

- **WHEN** `styles/Team` is a symlink to `shared/TeamStyle` that holds `Foo.yml`, and a case through the config flags `Team.Foo`
- **THEN** `hakkiri test --coverage styles/Team/Foo.yml` counts `Team.Foo` as covered

#### Scenario: A `.yaml` rule file

- **WHEN** `styles/Team/Other.yaml` holds `extends` and `tests`
- **THEN** `hakkiri test --coverage` reads no case from it and names no rule for it

#### Scenario: A nested rule

- **WHEN** the config has the package `styles` and a case loads `styles/Team/Sub/Bar.yml`
- **THEN** the rule is `Team.Sub.Bar`, and with no package that holds the file it is `Sub.Bar`, since the folder that holds the file is its style

#### Scenario: A dotted file name

- **WHEN** a case loads `styles/Team/Foo.v2.yml`
- **THEN** the rule is `Team.Foo`

#### Scenario: A project reached through a link

- **WHEN** the project root is reached through a symlink, a style folder is a symlink, and `test --coverage` runs on `link/styles/Team/Sub/Bar.yml`
- **THEN** the rule is `Team.Sub.Bar`, as `check` names it, and not `Sub.Bar`

#### Scenario: A name two rule files share

- **WHEN** `Terms.yml` and `Terms.custom.yml` both give `Team.Terms`, and a case flags it through either file
- **THEN** `test --coverage` counts the name as covered, as `vale test --coverage` does, and when no case flags it, lists it once with both files

#### Scenario: A shared name in JSON

- **WHEN** `test --coverage --format json` finds that no case flags `Team.Terms`, which `Terms.yml` and `Terms.custom.yml` both give
- **THEN** `uncovered` holds one object, `{"rule": "Team.Terms", "files": [...]}` with both files, and the object of a name one file gives holds `rule` alone
