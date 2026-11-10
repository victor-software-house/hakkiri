## ADDED Requirements

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
