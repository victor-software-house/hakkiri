## ADDED Requirements

### Requirement: A package glob can beat the project's top level, and the project can force a value

hakkiri SHALL rank a package's glob above the project's top level, and SHALL apply a value the project tags `!override` after every other value.  
Vale 66bbb85 applies the sections of a package before the project's in the order written, and has no way to force a value ([file.go][vale-file]).

#### Scenario: A package glob and the project's top level

- **WHEN** a package sets `'*.md': {code_span_size: 0}` and the project sets the top-level `code_span_size: 2`
- **THEN** hakkiri counts a code span in `README.md` as 0 words; Vale has no such key

#### Scenario: A forced value

- **WHEN** the project tags a value `!override`
- **THEN** hakkiri applies it after every value without the tag; Vale reads no YAML and forces nothing

[vale-file]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/file.go#L185-L215
