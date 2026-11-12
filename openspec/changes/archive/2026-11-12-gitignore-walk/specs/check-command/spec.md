## ADDED Requirements

### Requirement: A walk skips what git ignores

Inside a git repository, a walk SHALL skip each file and folder that `.gitignore`, `.git/info/exclude` and the global git excludes ignore, unless `gitignore` is false.  
It SHALL apply the `.gitignore` of each folder from the repository root down.  
A file named on the command line SHALL be linted whether git ignores it or not.

#### Scenario: An ignored folder and file

- **WHEN** `.gitignore` holds `build/` and `draft.md`, and `a.md`, `draft.md` and `build/b.md` each draw one warning
- **THEN** `hakkiri check` lints `a.md` alone, and prints `1 alert in 1 file`

#### Scenario: The exclude file and the global excludes

- **WHEN** the same two names are in `.git/info/exclude`, or in the global excludes file
- **THEN** `hakkiri check` lints `a.md` alone

#### Scenario: A nested gitignore

- **WHEN** `docs/.gitignore` holds `skip.md`, and `docs/skip.md` and `skip.md` exist
- **THEN** a walk skips `docs/skip.md` and lints `skip.md`

#### Scenario: A named ignored file

- **WHEN** `hakkiri check draft.md build/b.md` runs and git ignores both
- **THEN** hakkiri lints both

#### Scenario: Git rules off

- **WHEN** `--no-gitignore`, `--gitignore=false`, `HAKKIRI_GITIGNORE=false` or `gitignore: false` is set
- **THEN** a walk lints `a.md`, `build/b.md` and `draft.md`

### Requirement: A named file that no glob selects draws a warning

A file named on the command line that no glob of `files` matches SHALL draw a warning that names it, or a failure with exit 2 when `on_unmatched` is `error`, or nothing when it is `ignore`.  
A file that a glob matches whose styles hold no rule, a file found by a walk, and a file named twice SHALL NOT draw more than the one warning.

#### Scenario: A warning

- **WHEN** the only glob is `docs/*.md` and `hakkiri check README.md docs` runs
- **THEN** hakkiri prints `warning README.md: no glob selects it, so it is not linted` on standard error, lints `docs/a.md`, and exits 0

#### Scenario: An error

- **WHEN** `hakkiri check --on-unmatched error README.md` runs under the same config
- **THEN** hakkiri prints `failed README.md: no glob selects it, so it is not linted` and exits 2

#### Scenario: Ignored

- **WHEN** `--on-unmatched ignore` is given
- **THEN** hakkiri prints nothing for `README.md` and exits 0

#### Scenario: No rule to run

- **WHEN** a glob matches `CHANGELOG.md` with `styles: []` and `hakkiri check CHANGELOG.md` runs
- **THEN** hakkiri prints no warning

#### Scenario: A walk finds an unselected file

- **WHEN** `hakkiri check` walks a folder that holds `README.md`, which no glob selects
- **THEN** hakkiri prints no warning
