## ADDED Requirements

### Requirement: The run options are config keys

`fail_level`, `gitignore` and `on_unmatched` SHALL be keys of the project's config, with the values of `--fail-level`, `--gitignore` and `--on-unmatched`.  
Each SHALL be resolved as the flag, then its variable, then the config, then the default: `error`, `true` and `warn`.  
An unknown value SHALL be an error that names the line.

#### Scenario: The config sets the fail level

- **WHEN** the config holds `fail_level: warning` and a file draws one warning
- **THEN** `hakkiri check` exits 1

#### Scenario: The variable beats the config

- **WHEN** the same config runs with `HAKKIRI_FAIL_LEVEL=error`
- **THEN** `hakkiri check` exits 0

#### Scenario: The flag beats the variable

- **WHEN** `--fail-level warning` is given with `HAKKIRI_FAIL_LEVEL=error`
- **THEN** `hakkiri check` exits 1

#### Scenario: The same order for the other two

- **WHEN** `on_unmatched: error` is set, then `HAKKIRI_ON_UNMATCHED=ignore`, then `--on-unmatched warn`
- **THEN** a named file that no glob selects fails the run, then draws nothing, then draws a warning

#### Scenario: An unknown word

- **WHEN** the config holds `on_unmatched: loud` or `fail_level: loud`
- **THEN** hakkiri reports an error that names `loud`, and exits 2

#### Scenario: Not a boolean

- **WHEN** the config holds `gitignore: maybe`
- **THEN** hakkiri reports an error on line 1, and exits 2
