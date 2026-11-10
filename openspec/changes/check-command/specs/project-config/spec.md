## ADDED Requirements

### Requirement: Paths in the config are relative to the project root

The project root SHALL be the folder that holds the `.ctl` folder of the config file, or the folder of the config file when it is not in a `.ctl` folder.  
hakkiri SHALL read every `path` in the config relative to that root.

#### Scenario: A config named from another folder

- **WHEN** `HAKKIRI_CONFIG=../.ctl/hakkiri.yaml` is set in `other/` and `packages` holds `{path: styles}`
- **THEN** hakkiri loads the styles from `styles/` beside `.ctl`, not from `other/styles`

## RENAMED Requirements

- FROM: `### Requirement: A package is pinned by URL and hash`
- TO: `### Requirement: A package is a folder or a download`

## MODIFIED Requirements

### Requirement: A package is a folder or a download

Each entry in `packages` SHALL hold a `path` to a folder of styles, or a `url` and a `sha256`, and never both.  
It SHALL fetch and cache each `url` with no separate sync step, and lint with it only when its download matches the hash.  
It SHALL refuse two entries that hold a style of one name, and the error SHALL name both.

#### Scenario: A local folder

- **WHEN** `packages` holds `{path: styles}` and `styles/Team/Filler.yml` is a rule
- **THEN** a glob with `styles: [Team]` runs `Team.Filler`

#### Scenario: A matching package

- **WHEN** `packages` holds one entry whose download matches its `sha256`
- **THEN** hakkiri lints with the styles in that package, and a second run reads it from the cache

#### Scenario: A package that does not match

- **WHEN** a package's download does not match its `sha256`
- **THEN** hakkiri reports an error naming the `url`, lints nothing, and exits with a failure status

#### Scenario: An entry with both

- **WHEN** an entry holds a `path` and a `url`
- **THEN** hakkiri reports an error naming the entry's line, and exits with a failure status

#### Scenario: One style in two folders

- **WHEN** `packages` holds `{path: a}` and `{path: b}` and both hold a folder `Team`
- **THEN** hakkiri reports an error naming `a/Team` and `b/Team`, and exits with a failure status
