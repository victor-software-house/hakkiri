## ADDED Requirements

### Requirement: Settings resolve by specificity, where Vale applies its sections in order

hakkiri SHALL resolve the settings of the globs that match a file by the settings cascade of `project-config`, where Vale 66bbb85 applies its sections in the order written, a later section winning for the file ([file.go][vale-file]).

#### Scenario: A specific glob written first

- **WHEN** the config holds `'docs/*.md': {Team.Filler: error}` and then `'*.md': {Team.Filler: warning}`
- **THEN** hakkiri reports `Team.Filler` on `docs/a.md` as an error, and Vale, given the same two sections in that order, reports it as a warning

#### Scenario: A package glob and the project's top level

- **WHEN** a package sets `'*.md': {code_span_size: 0}` and the project sets the top-level `code_span_size: 2`
- **THEN** hakkiri counts a code span in `README.md` as 0 words; Vale has no such key, and its sections of a package apply before the project's in the order written

#### Scenario: A forced value

- **WHEN** the project tags a value `!override`
- **THEN** hakkiri applies it after every value without the tag; Vale has no way to force a value, and reads no YAML

[vale-file]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/file.go#L185-L215
