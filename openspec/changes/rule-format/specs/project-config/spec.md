# project-config

## Purpose

`.ctl/hakkiri.yaml` holds the rule packages a project pins, the lowest level it reports, and the styles and rule levels for each set of files.  
It holds the settings a `.vale.ini` holds, in lowercase keys.

## ADDED Requirements

### Requirement: hakkiri reads .ctl/hakkiri.yaml

hakkiri SHALL read its project config from `.ctl/hakkiri.yaml` in the current directory, or from the file that `--config` or `HAKKIRI_CONFIG` names, the flag winning over the variable.

#### Scenario: The variable names another file

- **WHEN** `HAKKIRI_CONFIG` is `docs/hakkiri.yaml` and no `--config` flag is given
- **THEN** hakkiri reads `docs/hakkiri.yaml` and not `.ctl/hakkiri.yaml`

### Requirement: An unknown key is refused

hakkiri SHALL refuse a project config that holds a key it does not define, and report an error that names the key and its line.

#### Scenario: A key in Vale's spelling

- **WHEN** `.ctl/hakkiri.yaml` holds `MinAlertLevel: warning` on line 4
- **THEN** hakkiri reports an error naming `MinAlertLevel` and line 4, and exits with a failure status

### Requirement: A package is pinned by URL and hash

Each entry in `packages` SHALL hold a `url` and a `sha256`.  
hakkiri SHALL fetch and cache each package with no separate sync step, and lint with it only when its download matches the hash.

#### Scenario: A matching package

- **WHEN** `packages` holds one entry whose download matches its `sha256`
- **THEN** hakkiri lints with the styles in that package, and a second run reads it from the cache

#### Scenario: A package that does not match

- **WHEN** a package's download does not match its `sha256`
- **THEN** hakkiri reports an error naming the `url`, lints nothing, and exits with a failure status

### Requirement: level sets the lowest level reported

hakkiri SHALL report only alerts at or above `level`, which is `suggestion`, `warning` or `error`, and SHALL report every level when `level` is unset.

#### Scenario: Suggestions are hidden

- **WHEN** `level` is `warning` and a file draws one suggestion and one warning
- **THEN** hakkiri reports the warning only

### Requirement: files maps globs to settings

hakkiri SHALL lint a file only when it matches a glob under `files`.  
It SHALL apply the settings of every glob the file matches, in the order written, a later glob winning for the same key.

#### Scenario: Two globs match one file

- **WHEN** `files` holds `'*.{md,mdx,rs}': {styles: [ProseStyle]}` and then `'*.rs': {ProseStyle.BareURL: off}`
- **THEN** a Rust file runs every ProseStyle rule except `ProseStyle.BareURL`, and a Markdown file runs every ProseStyle rule

#### Scenario: A file no glob matches

- **WHEN** `files` holds only `'*.md'` and the project has `notes.txt`
- **THEN** hakkiri does not lint `notes.txt`

### Requirement: styles lists the styles a glob runs

`styles` SHALL list the style names whose rules run on the files a glob matches.  
An empty list SHALL turn off linting for those files.

#### Scenario: Generated files are skipped

- **WHEN** `files` holds `'*.md': {styles: [ProseStyle]}` and then `CHANGELOG.md: {styles: []}`
- **THEN** hakkiri lints `README.md` and does not lint `CHANGELOG.md`

### Requirement: A rule name sets the rule's level

Under a glob, a key that names a rule SHALL set that rule's level for the matched files.  
The value SHALL be `suggestion`, `warning` or `error`, or `off` to turn the rule off.
A key that names no rule in any loaded style SHALL be refused, with an error that names the key and its line.

#### Scenario: A rule turned down

- **WHEN** a glob holds `Direct.Length: warning` and the rule's own level is `error`
- **THEN** hakkiri reports that rule's alerts on those files as warnings

#### Scenario: A misspelled rule name

- **WHEN** a glob holds `ProseStyle.BareUrl: off` on line 7, and no loaded style has a rule of that name
- **THEN** hakkiri reports an error naming `ProseStyle.BareUrl` and line 7, and exits with a failure status
