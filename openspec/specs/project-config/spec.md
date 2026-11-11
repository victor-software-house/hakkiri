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
A `url` SHALL start with `https://` or `http://`, and a `sha256` SHALL be 64 lowercase hex digits.  
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
- **THEN** hakkiri reports an error naming the `url`, the expected hash and the actual hash, lints nothing, caches nothing, and exits with a failure status

#### Scenario: An entry with both

- **WHEN** an entry holds a `path` and a `url`
- **THEN** hakkiri reports an error naming the entry's line, and exits with a failure status

#### Scenario: One style in two folders

- **WHEN** `packages` holds `{path: a}` and `{path: b}` and both hold a folder `Team`
- **THEN** hakkiri reports an error naming `a/Team` and `b/Team`, and exits with a failure status

#### Scenario: A folder of settings

- **WHEN** a package's `styles` folder holds `config/vocabularies/Team/accept.txt` beside the style `Team`
- **THEN** `config` is no style, a glob that lists `config` is an error, and two packages that each hold a `config` folder load together

#### Scenario: A hash of the wrong shape

- **WHEN** an entry holds `sha256: abc123`
- **THEN** hakkiri reports ``line 2: packages[0]: `sha256` is 64 lowercase hex digits``, and exits with a failure status

#### Scenario: A url that is not http

- **WHEN** an entry holds `url: file:///tmp/Team.zip`
- **THEN** hakkiri reports ``line 2: packages[0]: a package `url` starts with `https://` or `http://` ``, and exits with a failure status

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

### Requirement: A level or `on` runs a rule whichever style holds it

A rule name that gives a level SHALL run that rule on the matched files at that level, though no glob lists its style.  
`on` SHALL run the rule at the level of its rule file, and `off` SHALL stop a rule of a listed style and change nothing for a rule of a style no glob lists.

#### Scenario: A rule of a style no glob lists

- **WHEN** `files` holds `'a.md': {styles: [Other], Team.Filler: warning}`, the style `Team` also holds `Harsh`, and `a.md` holds `This is very terrible maybe bland.`
- **THEN** hakkiri reports `Team.Filler` as a warning on `very` and `Other.Bland` as a warning on `bland`, and runs no other rule of `Team`

#### Scenario: A rule setting and no styles

- **WHEN** `files` holds `'a.md': {Team.Hint: error}` and `a.md` holds `This is very terrible maybe bland.`
- **THEN** hakkiri reports one error, `Team.Hint` on `maybe`, though the rule's own level is `suggestion`, and exits 1

#### Scenario: A rule turned on at its own level

- **WHEN** `files` holds `'a.md': {styles: [Other], Team.Hint: on}`, and `a.md` holds `This is very terrible maybe bland.`
- **THEN** hakkiri reports `Team.Hint` as a suggestion, the level of its rule file, and `Other.Bland` as a warning, and exits 0

#### Scenario: A rule turned off in a style no glob lists

- **WHEN** `files` holds `'a.md': {styles: [Other], Team.Filler: off}`
- **THEN** hakkiri runs `Other` alone on `a.md`, with no error

#### Scenario: A style one glob lists and another glob enables by rule

- **WHEN** `files` holds `'*.md': {styles: [Other]}` and then `'a.md': {Team.Harsh: error}`
- **THEN** `a.md` runs `Other` and `Team.Harsh`, and `b.md` runs `Other` alone

### Requirement: Rule settings and style lists combine in the order written

A later glob SHALL win for the same rule, whether it turns the rule on or off or sets its level.  
A `styles` list SHALL replace the earlier list and keep the rule settings before it.  
An empty `styles` list SHALL drop every rule setting before it, and the rule settings of its own glob SHALL apply after it.  
The order of keys inside a glob SHALL NOT matter.

#### Scenario: A rule turned on after it was turned off

- **WHEN** `files` holds `'*.md': {styles: [Team], Team.Harsh: off}` and then `'a.md': {Team.Harsh: error}`
- **THEN** `a.md` runs `Team.Harsh` as an error, and `b.md` runs the rest of `Team`

#### Scenario: A rule turned off after it was enabled

- **WHEN** `files` holds `'*.md': {Team.Filler: warning}` and then `'a.md': {Team.Filler: off}`
- **THEN** `a.md` runs nothing, and `b.md` runs `Team.Filler` as a warning

#### Scenario: A rule turned back on

- **WHEN** `files` holds `'*.md': {styles: [Team], Team.Harsh: off}` and then `'a.md': {Team.Harsh: on}`
- **THEN** `a.md` runs `Team.Harsh` as an error, the level of its rule file, and `b.md` runs the rest of `Team`

#### Scenario: A level that stays

- **WHEN** `files` holds `'*.md': {Team.Hint: error}`, then `'a.md': {Team.Hint: off}`, then `'a?md': {Team.Hint: on}`
- **THEN** `a.md` and `b.md` report `Team.Hint` as an error

#### Scenario: A level set again

- **WHEN** `files` holds `'*.md': {Team.Filler: warning}` and then `'a.md': {Team.Filler: error}`
- **THEN** `a.md` reports `Team.Filler` as an error, and `b.md` as a warning

#### Scenario: A later list of styles

- **WHEN** `files` holds `'*.md': {Team.Filler: warning}` and then `'a.md': {styles: [Other]}`
- **THEN** `a.md` runs `Other` and `Team.Filler`

#### Scenario: An empty list of styles

- **WHEN** `files` holds `'*.md': {styles: [Other], Team.Filler: warning}`, then `'a.md': {styles: []}`, then `'b.md': {styles: []}` and `'b?md': {Team.Hint: warning}`
- **THEN** `a.md` runs nothing, `b.md` runs `Team.Hint` alone, and `c.md` runs `Other` and `Team.Filler`

#### Scenario: Keys in any order

- **WHEN** a glob holds `Team.Harsh: off` before `styles: [Team]`
- **THEN** the files it matches run `Team` without `Team.Harsh`

### Requirement: A rule setting does not make a clash an error

A rule setting SHALL NOT make hakkiri refuse a name clash or a bracket in the style of the rule it enables.  
When two files of a style that no glob lists give one name, and a setting enables that name, hakkiri SHALL run the later file in file name order.

#### Scenario: A clash in a style a setting enables

- **WHEN** `Team/Terms.yml` and `Team/Terms.custom.yml` both flag `very`, and `files` holds `'*.md': {styles: [Other], Team.Terms: warning}`
- **THEN** hakkiri accepts the config, and `This is very good.` draws one alert from `Team/Terms.yml`, the later file

### Requirement: A download is checked before it is read

A body SHALL be at most 32 MiB and SHALL match the `sha256`.  
It SHALL be a zip with one top folder that names no file outside the package and unpacks to at most 128 MiB.  
The top folder SHALL be a style, or hold a `styles` folder of styles, as in a Vale package.  
Each fault SHALL be an error that names the `url`, and a download that fails a check SHALL leave nothing in the cache.

#### Scenario: A server answers 404

- **WHEN** a `url` answers `404`
- **THEN** hakkiri reports an error naming the `url` and `404`, and exits with a failure status

#### Scenario: A redirect

- **WHEN** a `url` answers `302` with a `Location` to a zip that matches the hash
- **THEN** hakkiri follows it and lints with the package

#### Scenario: A body that is not a zip

- **WHEN** a body matches its hash and is not a zip
- **THEN** hakkiri reports an error naming the `url` and saying it is not a zip

#### Scenario: A name that leaves the package

- **WHEN** a zip holds the entry `../escape.txt`
- **THEN** hakkiri reports an error naming the `url` and the entry, writes no file outside the cache folder of that package, and caches nothing

#### Scenario: A zip with no top folder

- **WHEN** a zip holds a file at its root and no folder
- **THEN** hakkiri reports an error naming the `url` and saying a package holds one top folder, and caches nothing

#### Scenario: A style-only package

- **WHEN** a zip holds `Team/Filler.yml` and no `styles` folder
- **THEN** a glob with `styles: [Team]` runs `Team.Filler`

#### Scenario: A Vale package is kept whole

- **WHEN** a package holds `Team/.vale.ini` and `Team/styles/Team/Filler.yml`
- **THEN** the cache folder of its hash holds both files where the zip put them

#### Scenario: A body over the limit

- **WHEN** a body is over 32 MiB
- **THEN** hakkiri reports an error naming the `url` and the limit, and caches nothing

### Requirement: A fetched package is cached by its hash

hakkiri SHALL keep each fetched package in the folder `<cache>/<sha256>/`, and SHALL read it with no request when that folder exists.

#### Scenario: A second run

- **WHEN** two runs use one package and one cache folder
- **THEN** the server sees one request

#### Scenario: Two runs fetch at once

- **WHEN** two runs fetch one package into one cache folder at the same moment, even with one process id
- **THEN** both succeed, the folder `<cache>/<sha256>/` holds one complete package, and the cache holds no unpacking folder

#### Scenario: A cached package with the server gone

- **WHEN** a package is cached and its server no longer answers
- **THEN** `check` lints with it and prints the alerts it printed before

#### Scenario: An empty cache with the server gone

- **WHEN** the cache holds no folder for the hash and the server does not answer
- **THEN** hakkiri reports an error naming the `url`, lints nothing, and exits with a failure status

#### Scenario: The test command

- **WHEN** a case runs through the config and the config pins a package by `url`
- **THEN** `hakkiri test` loads the fetched package and the case passes

### Requirement: The cache folder is an option

`--cache-dir`, or the variable `HAKKIRI_CACHE_DIR`, SHALL name the cache folder, and its default SHALL be the per-user cache folder `hakkiri`.

#### Scenario: A cache folder from the variable

- **WHEN** `HAKKIRI_CACHE_DIR` names a folder and a package is fetched
- **THEN** the package is in that folder under its hash

#### Scenario: The default folder

- **WHEN** neither the option nor the variable is set
- **THEN** the cache is `~/Library/Caches/hakkiri` on macOS, and `$XDG_CACHE_HOME/hakkiri` or `~/.cache/hakkiri` elsewhere

### Requirement: A rule hakkiri cannot run fails the run until it is turned off

Each rule hakkiri cannot run that the config runs on a linted file SHALL make the run exit 2, with a `failed` line that names the rule, its file and the setting that leaves it out.  
A rule is run when its style is listed and no setting turns it `off`, or when a level or `on` turns it on.  
A rule in a style no glob lists, and that no level or `on` names, SHALL be ignored.  
`off` SHALL leave it out, and a level or `on` SHALL NOT make it work: the run still fails for it.

#### Scenario: A script rule in a listed style

- **WHEN** `Kid/Cannot.yml` holds `extends: script`, `Kid/Parent.yml` flags `alpha`, `beta` and `gamma`, `files` holds `'*.md': {styles: [Kid]}`, and `a.md` holds `alpha beta gamma delta swapme`
- **THEN** hakkiri reports three alerts from `Kid.Parent`, prints one `failed` line for `Kid.Cannot`, and exits 2

#### Scenario: A doc selector

- **WHEN** a rule of a listed style has `scope: doc(h1)`
- **THEN** hakkiri prints the same `failed` line for it, and runs the rest of the style

#### Scenario: A rule turned off

- **WHEN** the glob also holds `Kid.Cannot: off`
- **THEN** hakkiri reports the three alerts, prints no `failed` line, and exits 0

#### Scenario: A style no glob lists

- **WHEN** the glob lists `Other` and not `Kid`
- **THEN** `Kid.Cannot` is ignored and the run exits 0

#### Scenario: A level or on

- **WHEN** a glob that lists `Other` holds `Kid.Cannot: warning`, or `Kid.Cannot: on`
- **THEN** hakkiri prints the `failed` line for `Kid.Cannot`, and exits 2

#### Scenario: A rule that extends a script rule

- **WHEN** a rule of a listed style extends a script rule
- **THEN** hakkiri prints a `failed` line for it, as for the script rule

### Requirement: The config names the vocabularies a project uses

The project config SHALL name vocabularies by the name of their folder in a `vocab` list, at its top level and in a glob of `files`, and hakkiri SHALL read each from a loaded package before it builds any rule.

#### Scenario: A vocabulary of a package

- **WHEN** `packages` holds `{path: styles}`, `styles/config/vocabularies/Team/accept.txt` holds `GitHub`, and the top level holds `vocab: [Team]`
- **THEN** `GitHub` is an accepted term of every rule that reads one

#### Scenario: A glob's vocabulary

- **WHEN** `files` holds `'docs/*.md': {vocab: [Docs]}` and `Docs` accepts `Zebra`
- **THEN** `Zebra` is accepted in the files that glob matches, and in no other

#### Scenario: No vocab key

- **WHEN** the config holds no `vocab` key
- **THEN** no accepted or rejected term exists, and every rule holds back what it held back before

#### Scenario: A vocabulary of a fetched package

- **WHEN** a package pinned by `url` and `sha256` holds `styles/config/vocabularies/Team/`, and `vocab` names `Team`
- **THEN** hakkiri reads the lists from the cache folder of that package, and a second run makes no request

#### Scenario: A vocab list that is not a list

- **WHEN** `vocab` holds the word `Team` and not a list
- **THEN** hakkiri reports an error on the line of `vocab`, and exits with a failure status
