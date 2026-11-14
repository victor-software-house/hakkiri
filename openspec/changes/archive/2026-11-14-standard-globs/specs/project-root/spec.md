## ADDED Requirements

### Requirement: A glob follows the rules of gitignore

A `files` or `formats` glob SHALL match as gitignore matches a pattern.  
`*`, `?` and a class SHALL stay inside one folder, and `**` SHALL span folders.  
A glob with no `/` SHALL match the file name in any folder, a leading `/` SHALL tie a glob to the project root, and a trailing `/` SHALL match every file the folder holds.  
A trailing `/**` SHALL match only the files below its folder.

#### Scenario: A star inside one folder

- **WHEN** `files` holds `'docs/*.md'`, and the project holds `docs/a.md` and `docs/x/a.md`
- **THEN** hakkiri lints `docs/a.md` and not `docs/x/a.md`

#### Scenario: A glob with no slash

- **WHEN** `files` holds `'*.md'`, and the project holds `a.md` and `docs/x/a.md`
- **THEN** hakkiri lints both

#### Scenario: A trailing double star

- **WHEN** `files` holds `'tests/oracle/*/**'`, and the project holds `tests/oracle/a.md` and `tests/oracle/x/a.md`
- **THEN** hakkiri lints `tests/oracle/x/a.md` and not `tests/oracle/a.md`

#### Scenario: A leading slash

- **WHEN** `files` holds `'/a.md'`, and the project holds `a.md` and `docs/a.md`
- **THEN** hakkiri lints `a.md` and not `docs/a.md`
