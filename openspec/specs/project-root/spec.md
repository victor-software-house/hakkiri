# project-root Specification

## Purpose

The project root is the folder that contains the `.ctl` folder of the config.
Globs and package paths are read from it, so a file matches the same glob however it is reached.
A path outside it draws a warning.

## Requirements

### Requirement: A glob matches the path under the project root

hakkiri SHALL match each `files` glob against the file's path relative to the project root, with `/` separators.
It SHALL resolve the root and the folders of the file through symlinks, and keep the file's own name.
So a symlink to a file matches by the name it is reached by, as in Vale.

#### Scenario: A dot-slash argument

- **WHEN** `files` contains `'docs/**'` and `hakkiri check ./docs` runs
- **THEN** hakkiri lints `docs/a.md` with the styles of that glob

#### Scenario: An absolute argument

- **WHEN** `files` contains `'docs/**'` and `hakkiri check` is given the absolute path of `docs/a.md`
- **THEN** hakkiri lints it with the styles of that glob

#### Scenario: A parent argument

- **WHEN** `files` contains `'docs/**'` and `HAKKIRI_CONFIG=../.ctl/hakkiri.yaml hakkiri check ../docs/a.md` runs in `other/`
- **THEN** hakkiri lints `docs/a.md` with the styles of that glob

#### Scenario: A root reached through a symlink

- **WHEN** `link` is a symlink to the project folder and `hakkiri check link/in.md` runs with the config `link/.ctl/hakkiri.yaml`
- **THEN** hakkiri lints `in.md` with the styles of its glob and prints no warning

#### Scenario: A symlink to a file

- **WHEN** `files` contains `'docs/**'` and `docs/a.md` is a symlink to `shared/a.md`, or to a file outside the root
- **THEN** hakkiri lints it as `docs/a.md`, with the styles of that glob

### Requirement: A path outside the project root is reported

hakkiri SHALL print a warning on standard error for each path it is given that is outside the project root and does not contain it.
It SHALL lint none of the files of that path.
When it is given no path, the current directory is the path given.

When no path lies under the root or contains it, hakkiri SHALL report a failure that gives the root and exit 2.

#### Scenario: One file outside the root

- **WHEN** `hakkiri check in.md ../outside.md` runs in `project/`, which contains the root, and `in.md` draws a warning
- **THEN** hakkiri prints a warning that shows `../outside.md`, lints `in.md`, and exits 0

#### Scenario: No file under the root

- **WHEN** `hakkiri check ../outside.md` runs in `project/`
- **THEN** hakkiri prints a warning that shows `../outside.md`, reports a failure that shows the root, and exits 2

#### Scenario: A config elsewhere

- **WHEN** `HAKKIRI_CONFIG=../other/hakkiri.yaml hakkiri check` runs in `project/`, so the root is `other/`
- **THEN** hakkiri prints a warning that shows `.`, reports a failure that shows the root, and exits 2

### Requirement: The warning is a diagnostic

hakkiri SHALL write the warning to standard error in every output format.
`--quiet` SHALL not hide it, because `--quiet` hides only a successful result.

A run with some paths under the root and some outside SHALL lint those under it and warn for each one outside.
It SHALL exit by the alert level.
The failure that gives the root SHALL be one of the `failures` of the JSON object.

#### Scenario: A mixed run

- **WHEN** `hakkiri check --fail-level warning in.md ../outside.md` runs in `project/` and `in.md` draws a warning
- **THEN** hakkiri warns for `../outside.md`, lints `in.md`, and exits 1

#### Scenario: JSON output

- **WHEN** `hakkiri check --format json ../outside.md` runs in `project/`
- **THEN** standard error contains the warning, standard output is one JSON object whose `failures` shows the root, and hakkiri exits 2

#### Scenario: Quiet output

- **WHEN** `hakkiri check --quiet in.md ../outside.md` runs in `project/`
- **THEN** standard output is empty, standard error contains the warning, and hakkiri exits 0

### Requirement: A walk enters only the way to the project root

A walk of a directory that contains the root SHALL enter only the folders on the way to the root and the folders under it.

#### Scenario: A walk from a folder that contains the root

- **WHEN** `hakkiri check ..` runs in `project/` and a sibling folder `sibling/locked` cannot be read
- **THEN** hakkiri does not enter `sibling`, lints the files under the root, and exits 0

### Requirement: A walk compares resolved paths

When it skips package folders, a walk SHALL compare the resolved path of each folder with the resolved package paths.
So a folder written in another case, or below a symlinked folder, is skipped alike.

A walk SHALL NOT enter a symlink to a folder.
A directory given as an argument SHALL be walked even when it is a package folder, as Vale lints its styles path only when given.

#### Scenario: A symlink to a package folder

- **WHEN** `docs/styles` is a symlink to the package folder `styles/`, and `hakkiri check docs` runs
- **THEN** hakkiri does not enter `docs/styles`

#### Scenario: A package folder given as an argument

- **WHEN** `hakkiri check styles` runs and `styles/Team/notes.md` is prose a glob matches
- **THEN** hakkiri lints `notes.md`

### Requirement: A missing path is reported once

A path that does not exist SHALL be a failure of its own, with no failure about the root.

#### Scenario: A path that does not exist

- **WHEN** `hakkiri check typo.md` runs and `typo.md` does not exist
- **THEN** hakkiri reports a failure that shows `typo.md` and no failure about the root, and exits 2

### Requirement: A glob follows the rules of gitignore

A `files` or `formats` glob SHALL match as gitignore matches a pattern.
`*`, `?` and a class SHALL stay inside one folder, and `**` SHALL span folders.

A glob with no `/` SHALL match the file name in any folder, and a leading `/` SHALL tie a glob to the project root.
With a trailing `/`, it SHALL match every file the folder contains.
A trailing `/**` SHALL match only the files below its folder.

#### Scenario: A star inside one folder

- **WHEN** `files` contains `'docs/*.md'`, and the project contains `docs/a.md` and `docs/x/a.md`
- **THEN** hakkiri lints `docs/a.md` and not `docs/x/a.md`

#### Scenario: A glob with no slash

- **WHEN** `files` contains `'*.md'`, and the project contains `a.md` and `docs/x/a.md`
- **THEN** hakkiri lints both

#### Scenario: A trailing double star

- **WHEN** `files` contains `'tests/oracle/*/**'`, and the project contains `tests/oracle/a.md` and `tests/oracle/x/a.md`
- **THEN** hakkiri lints `tests/oracle/x/a.md` and not `tests/oracle/a.md`

#### Scenario: A leading slash

- **WHEN** `files` contains `'/a.md'`, and the project contains `a.md` and `docs/a.md`
- **THEN** hakkiri lints `a.md` and not `docs/a.md`
