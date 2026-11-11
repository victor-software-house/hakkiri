# project-config Specification

## Purpose

`.ctl/hakkiri.yaml` holds the rule packages a project pins, the lowest level it reports, and the styles and rule levels for each set of files.  
It holds the settings a `.vale.ini` holds, in lowercase keys.

## Requirements

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

hakkiri SHALL refuse a key other than `styles` that names no rule in any style the config loads.  
Its error SHALL name the key and its line.  
A rule name SHALL match case-sensitively.

#### Scenario: A rule turned down

- **WHEN** a glob holds `Direct.Length: warning` and the rule's own level is `error`
- **THEN** hakkiri reports that rule's alerts on those files as warnings

#### Scenario: A misspelled rule name

- **WHEN** a glob holds `ProseStyle.BareUrl: off` on line 7, and the loaded ProseStyle has a rule `BareURL` and none named `BareUrl`
- **THEN** hakkiri reports an error naming `ProseStyle.BareUrl` and line 7, and exits with a failure status

#### Scenario: A rule from a style another glob lists

- **WHEN** `files` holds `'*.{md,rs}': {styles: [ProseStyle]}` and then `'*.rs': {ProseStyle.BareURL: off}`, and ProseStyle has a rule `BareURL`
- **THEN** hakkiri accepts the config, though the `'*.rs'` glob lists no `styles` of its own

### Requirement: Paths in the config are relative to the project root

The project root SHALL be the folder that holds the `.ctl` folder of the config file, or the folder of the config file when it is not in a `.ctl` folder, with symlinks resolved.  
hakkiri SHALL read every `path` in the config relative to that root, and a walk SHALL compare each folder's resolved path with those resolved package paths when it skips package folders.

A `packages` `path` that does not exist SHALL be an error that names its line, and hakkiri SHALL NOT fall back to a path as written.

#### Scenario: A package folder that is missing

- **WHEN** `packages` holds `{path: styles}` and `styles/` does not exist
- **THEN** hakkiri reports an error naming the line of that `path` and exits 2

#### Scenario: A config named from another folder

- **WHEN** `HAKKIRI_CONFIG=../.ctl/hakkiri.yaml` is set in `other/` and `packages` holds `{path: styles}`
- **THEN** hakkiri loads the styles from `styles/` beside `.ctl`, not from `other/styles`
