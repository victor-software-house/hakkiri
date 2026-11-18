## ADDED Requirements

### Requirement: A package's settings do not set the run options

A package's settings file SHALL NOT contain `fail_level`, `gitignore` or `on_unmatched`.
A file that does SHALL be an error that gives the key, its line and the package.

#### Scenario: A package sets all three

- **WHEN** a package's `.ctl/hakkiri.yaml` contains `gitignore: false` on line 2, `fail_level: warning` on line 3 and `on_unmatched: error` on line 4
- **THEN** hakkiri reports an error for each, such as `pkg/.ctl/hakkiri.yaml: line 2: gitignore: belongs to the project's config`, and exits 2
