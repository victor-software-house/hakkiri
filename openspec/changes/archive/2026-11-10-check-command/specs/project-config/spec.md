## ADDED Requirements

### Requirement: Paths in the config are relative to the project root

The project root SHALL be the folder that contains the `.ctl` folder of the config file, with symlinks resolved.  
When the config file is not in a `.ctl` folder, the root SHALL be the folder of the config file, with symlinks resolved.

hakkiri SHALL read every `path` in the config relative to that root.  
When a walk skips the package folders, it SHALL compare each folder's resolved path with those resolved package paths.

A `packages` `path` that does not exist SHALL be an error that names its line, and hakkiri SHALL NOT fall back to a path as written.

#### Scenario: A package folder that is missing

- **WHEN** `packages` contains `{path: styles}` and `styles/` does not exist
- **THEN** hakkiri reports an error naming the line of that `path` and exits 2

#### Scenario: A config named from another folder

- **WHEN** `HAKKIRI_CONFIG=../.ctl/hakkiri.yaml` is set in `other/` and `packages` contains `{path: styles}`
- **THEN** hakkiri loads the styles from `styles/` beside `.ctl`, not from `other/styles`

## RENAMED Requirements

A rename lists FROM (the old heading) and TO (the new heading).

- FROM: `### Requirement: A package is pinned by URL and hash`
- TO: `### Requirement: A package is a folder or a download`

## MODIFIED Requirements

### Requirement: A package is a folder or a download

Each entry in `packages` SHALL contain a `path` to a folder of styles, or a `url` and a `sha256`, and never both.  
It SHALL fetch and cache each `url` with no separate sync step, and lint with it only when its download matches the hash.

It SHALL refuse two entries that contain a style of one name, and the error SHALL show both names.

#### Scenario: A local folder

- **WHEN** `packages` contains `{path: styles}` and `styles/Team/Filler.yml` is a rule
- **THEN** a glob with `styles: [Team]` runs `Team.Filler`

#### Scenario: A matching package

- **WHEN** `packages` contains one entry whose download matches its `sha256`
- **THEN** hakkiri lints with the styles in that package, and a second run reads it from the cache

#### Scenario: A package that does not match

- **WHEN** a package's download does not match its `sha256`
- **THEN** hakkiri reports an error naming the `url`, lints nothing, and exits with a failure status

#### Scenario: An entry with both

- **WHEN** an entry contains a `path` and a `url`
- **THEN** hakkiri reports an error naming the entry's line, and exits with a failure status

#### Scenario: One style in two folders

- **WHEN** `packages` contains `{path: a}` and `{path: b}` and both contain a folder `Team`
- **THEN** hakkiri reports an error naming `a/Team` and `b/Team`, and exits with a failure status
