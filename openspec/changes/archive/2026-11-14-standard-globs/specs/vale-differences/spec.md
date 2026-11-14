## ADDED Requirements

### Requirement: A glob follows gitignore, where Vale matches more

hakkiri SHALL match a glob by the rules of gitignore, as the `project-root` spec sets.  
Vale 66bbb85 reads a glob with no `**` through gobwas with no separator, so its `*` crosses `/`.  
It reads a glob with `**` through doublestar, whose trailing `/**` also matches zero folders.

#### Scenario: A star across folders

- **WHEN** a glob is `docs/*.md` and the project holds `docs/x/a.md`
- **THEN** Vale lints the file, and hakkiri does not ([decision 1][standard-globs-1])

#### Scenario: A trailing double star

- **WHEN** a glob is `tests/oracle/*/**` and the project holds `tests/oracle/a.md`
- **THEN** Vale lints the file, and hakkiri does not ([decision 1][standard-globs-1])

[standard-globs-1]: ../../design.md#1-globs-follow-gitignore
