## ADDED Requirements

### Requirement: A package's settings hold formats and not on_unknown

A package's settings file MAY contain `formats`, and the entries of the packages and the project SHALL add up.
A later entry SHALL replace an earlier one with the same key.

A package's settings file SHALL NOT contain `on_unknown`.
A file that does SHALL be an error that gives the key, its line and the package.

#### Scenario: The project overrides the package

- **WHEN** a package maps `txt: md` and the project maps `txt: text`
- **THEN** `a.txt` is read as plain text, and read as Markdown when the project has no entry

#### Scenario: A package sets on_unknown

- **WHEN** a package's file contains `on_unknown: warn` on line 5
- **THEN** hakkiri reports `pkg/.ctl/hakkiri.yaml: line 5: on_unknown: belongs to the project's config` and exits 2
