## ADDED Requirements

### Requirement: A package's settings do not set the run options

A package's settings file SHALL NOT hold `fail_level`, `gitignore` or `on_unmatched`, and a file that does SHALL be an error that names the key, its line and the package.

#### Scenario: A package sets all three

- **WHEN** a package's `.ctl/hakkiri.yaml` holds `gitignore: false` on line 2, `fail_level: warning` on line 3 and `on_unmatched: error` on line 4
- **THEN** hakkiri reports an error for each, such as `pkg/.ctl/hakkiri.yaml: line 2: gitignore: belongs to the project's config`, and exits 2
