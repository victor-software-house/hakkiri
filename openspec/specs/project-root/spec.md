# project-root Specification

## Purpose

The project root is the folder that holds the `.ctl` folder of the config.  
Globs and package paths are read from it, so a file matches the same glob however it is named, and a path outside it draws a warning.

## Requirements

### Requirement: A glob matches the path under the project root

hakkiri SHALL match each `files` glob against the file's path relative to the project root, with `/` separators.  
It SHALL resolve the root and the folders of the file through symlinks, and keep the file's own name, so a symlink to a file matches by the name it is reached by, as in Vale.

#### Scenario: A dot-slash argument

- **WHEN** `files` holds `'docs/**'` and `hakkiri check ./docs` runs
- **THEN** hakkiri lints `docs/a.md` with the styles of that glob

#### Scenario: An absolute argument

- **WHEN** `files` holds `'docs/**'` and `hakkiri check` is given the absolute path of `docs/a.md`
- **THEN** hakkiri lints it with the styles of that glob

#### Scenario: A parent argument

- **WHEN** `files` holds `'docs/**'` and `HAKKIRI_CONFIG=../.ctl/hakkiri.yaml hakkiri check ../docs/a.md` runs in `other/`
- **THEN** hakkiri lints `docs/a.md` with the styles of that glob

#### Scenario: A root reached through a symlink

- **WHEN** `link` is a symlink to the project folder and `hakkiri check link/in.md` runs with the config `link/.ctl/hakkiri.yaml`
- **THEN** hakkiri lints `in.md` with the styles of its glob and prints no warning

#### Scenario: A symlink to a file

- **WHEN** `files` holds `'docs/**'` and `docs/a.md` is a symlink to `shared/a.md`, or to a file outside the root
- **THEN** hakkiri lints it as `docs/a.md`, with the styles of that glob

### Requirement: A path outside the project root is reported

hakkiri SHALL print a warning on standard error for each path it is given that is outside the project root and does not hold it, and SHALL lint none of its files.  
When it is given no path, the current directory is the path given.  
When no path lies under the root or holds it, hakkiri SHALL report a failure that names the root and exit 2.  

#### Scenario: One file outside the root

- **WHEN** `hakkiri check in.md ../outside.md` runs in `project/`, which holds the root, and `in.md` draws a warning
- **THEN** hakkiri prints a warning that names `../outside.md`, lints `in.md`, and exits 0

#### Scenario: No file under the root

- **WHEN** `hakkiri check ../outside.md` runs in `project/`
- **THEN** hakkiri prints a warning that names `../outside.md`, reports a failure naming the root, and exits 2

#### Scenario: A config elsewhere

- **WHEN** `HAKKIRI_CONFIG=../other/hakkiri.yaml hakkiri check` runs in `project/`, so the root is `other/`
- **THEN** hakkiri prints a warning that names `.`, reports a failure naming the root, and exits 2

### Requirement: The warning is a diagnostic

hakkiri SHALL write the warning to standard error in every output format, and `--quiet` SHALL not hide it, because `--quiet` hides only a successful result.  
A run with some paths under the root and some outside SHALL lint those under it, warn for each one outside, and exit by the alert level.  
The failure that names the root SHALL be one of the `failures` of the JSON object.

#### Scenario: A mixed run

- **WHEN** `hakkiri check --fail-level warning in.md ../outside.md` runs in `project/` and `in.md` draws a warning
- **THEN** hakkiri warns for `../outside.md`, lints `in.md`, and exits 1

#### Scenario: JSON output

- **WHEN** `hakkiri check --format json ../outside.md` runs in `project/`
- **THEN** standard error holds the warning, standard output is one JSON object whose `failures` names the root, and hakkiri exits 2

#### Scenario: Quiet output

- **WHEN** `hakkiri check --quiet in.md ../outside.md` runs in `project/`
- **THEN** standard output is empty, standard error holds the warning, and hakkiri exits 0

### Requirement: A walk enters only the way to the project root

A walk of a directory that holds the root SHALL enter only the folders on the way to the root and the folders under it.

#### Scenario: A walk from a folder that holds the root

- **WHEN** `hakkiri check ..` runs in `project/` and a sibling folder `sibling/locked` cannot be read
- **THEN** hakkiri does not enter `sibling`, lints the files under the root, and exits 0

### Requirement: A walk compares resolved paths

When it skips package folders, a walk SHALL compare the resolved path of each folder with the resolved package paths, so a folder written in another case, or below a symlinked folder, is skipped alike.  
A walk SHALL NOT enter a symlink to a folder.  
A directory named as an argument SHALL be walked even when it is a package folder, as Vale lints its styles path only when named.

#### Scenario: A symlink to a package folder

- **WHEN** `docs/styles` is a symlink to the package folder `styles/`, and `hakkiri check docs` runs
- **THEN** hakkiri does not enter `docs/styles`

#### Scenario: A package folder named as an argument

- **WHEN** `hakkiri check styles` runs and `styles/Team/notes.md` is prose a glob matches
- **THEN** hakkiri lints `notes.md`

### Requirement: A missing path is reported once

A path that does not exist SHALL be a failure of its own, with no failure about the root.

#### Scenario: A path that does not exist

- **WHEN** `hakkiri check typo.md` runs and `typo.md` does not exist
- **THEN** hakkiri reports a failure naming `typo.md` and no failure about the root, and exits 2
