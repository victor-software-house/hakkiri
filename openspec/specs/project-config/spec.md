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
It SHALL apply the settings of every glob the file matches in the rank of the settings cascade, a higher rank winning for the same key.

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

### Requirement: A package may ship a settings file

hakkiri SHALL read `.ctl/hakkiri.yaml` in the top folder of each loaded package, when it exists, in the format of the project config.  
It SHALL apply the settings of each package before the project's own, in the order of `packages`, as if they were written ahead of the project's `files` entries.  
A package with no such file SHALL change nothing.

#### Scenario: A package turns a style on

- **WHEN** the package `pkg` holds `styles/Team/Filler.yml` flagging `very` and `.ctl/hakkiri.yaml` with `files: {'*.md': {styles: [Team]}}`, the project config is `packages: [{path: pkg/styles}]` with no `files`, and `a.md` holds `This is very good.`
- **THEN** hakkiri reports one warning from `Team.Filler` in `a.md`

#### Scenario: A package with no settings file

- **WHEN** the same package holds no `.ctl/hakkiri.yaml`
- **THEN** hakkiri lints no file, as it does with a project that has no `files`

### Requirement: The project overrides a package

At equal specificity a setting in the project's config SHALL win over the same setting in a package's file, and a later package SHALL win over an earlier one.  
A rule that a package sets `off` SHALL stay off until a setting of a glob that ranks higher turns it on.  
The project's `level` SHALL win, and a package's `level` SHALL apply only when the project sets none.

#### Scenario: A project turns off a package rule

- **WHEN** a package sets `'*.md': {styles: [Team], Team.Filler: warning}`, and the project holds `files: {'a.md': {Team.Filler: off}}`
- **THEN** `a.md` draws no alert from `Team.Filler`, and `b.md` draws one

#### Scenario: A package rule that ships off

- **WHEN** a package sets `'*.md': {styles: [Team], Team.Filler: off}`, and `a.md` holds `This is very good.`
- **THEN** hakkiri reports nothing, and the project setting `files: {'*.md': {Team.Filler: on}}` makes it report one warning

#### Scenario: A project level

- **WHEN** a package sets `level: error` and the project sets `level: suggestion`
- **THEN** hakkiri reports from the suggestion level up

#### Scenario: A package level the project leaves alone

- **WHEN** a package sets `level: error` and the project sets no `level`
- **THEN** hakkiri reports only errors

#### Scenario: Two packages

- **WHEN** the packages `a` and `b`, in that order, both set `'*.md': {Team.Filler: ...}` to a level
- **THEN** the level of `b` applies, and with the order reversed the level of `a` applies

### Requirement: A project's styles join those of a package under one glob

A project's `styles` list under the same glob text as a package's SHALL add to the package's list.  
A `styles` list under another glob SHALL compete in the settings cascade, the list of the higher rank replacing the other.

#### Scenario: The same glob

- **WHEN** a package sets `'*.md': {styles: [Team]}` and the project sets `'*.md': {styles: [Other]}`
- **THEN** a Markdown file runs `Team` and `Other`

#### Scenario: Another glob

- **WHEN** a package sets `'*.md': {styles: [Team]}` and the project sets `'docs/*.md': {styles: [Other]}`
- **THEN** `docs/a.md` runs `Other`, the rule settings of the package still apply to it, and `a.md` runs `Team`

#### Scenario: An empty list

- **WHEN** the project sets `'*.md': {styles: []}` after a package's `'*.md': {styles: [Team]}`
- **THEN** a Markdown file runs no style from the package

### Requirement: A package's settings file holds the keys of a project config

A package's settings file SHALL hold `files`, `level`, `vocab` and `packages`.  
A key it holds that the project config does not define SHALL be an error that names the key, its line and the package.  
A rule setting that names no loaded rule SHALL be an error, as in the project's config.

#### Scenario: A key in Vale's spelling

- **WHEN** a package's `.ctl/hakkiri.yaml` holds `MinAlertLevel: warning` on line 3
- **THEN** hakkiri reports an error naming `MinAlertLevel`, line 3 and the package, and exits 2

#### Scenario: A rule the package does not hold

- **WHEN** a package's file sets `Team.Nope: warning` and no loaded style holds `Team.Nope`
- **THEN** hakkiri reports an error naming the key and the line

### Requirement: The built-in style Vale holds the vocabulary rules

hakkiri SHALL accept `Vale` in a glob's `styles` as a style whose rules are `Repetition` and `Spelling`, and `Terms` and `Avoid` when a named vocabulary at the top level holds a term.
The style SHALL exist whatever the vocabularies hold.

#### Scenario: A style named Vale

- **WHEN** a glob lists `styles: [Vale]` and the config names a vocabulary with an accepted term
- **THEN** the glob runs `Vale.Repetition`, `Vale.Spelling` and `Vale.Terms`

#### Scenario: No vocabulary

- **WHEN** a glob lists `styles: [Vale]` and no named vocabulary holds a term
- **THEN** the glob runs `Vale.Repetition` and `Vale.Spelling`

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

### Requirement: formats maps a file to a reader

`formats` SHALL map an extension, a file name or a glob to a reader, named by any langdex name, alias or extension of the language it reads: Markdown, MDX, Rust, TypeScript, TSX, JavaScript, Go, Swift, YAML, TOML, Shell or Text.  
So `md`, `markdown` and `Markdown` name one reader, `sh` and `bash` name the Shell reader, and `text` names plain text.  
An entry SHALL beat the language of the file.

#### Scenario: An extension

- **WHEN** `formats` holds `txt: md`
- **THEN** `a.txt` is read as Markdown

#### Scenario: Another name of the reader

- **WHEN** `formats` holds `txt: markdown`
- **THEN** `a.txt` is read as Markdown

#### Scenario: Over the table

- **WHEN** `formats` holds `md: text`
- **THEN** `a.md` is read as plain text

#### Scenario: A script with no extension

- **WHEN** `formats` holds `mise-tasks/*: sh`
- **THEN** a file `mise-tasks/lint` is read as Shell

### Requirement: formats entries are looked up in key order

An entry named by the file's extension SHALL come first, then, in the order of the keys as text, an entry whose key is the file's name or a glob that matches its name or its path.  
A key that holds `*`, `?`, `[`, `{` or `/` SHALL be a glob.

#### Scenario: A name and a glob

- **WHEN** `formats` holds `NOTES: md` and `conf/*.cfg: md`
- **THEN** `NOTES` and `conf/x.cfg` are read as Markdown, and `other.cfg` as plain text

#### Scenario: The extension first

- **WHEN** `formats` holds `txt: text` and `'*': md`
- **THEN** `a.txt` is read as plain text and `b.cfg` as Markdown

### Requirement: formats refuses a bad entry

A reader that names no language hakkiri reads SHALL be an error that lists the twelve, and a key that holds a glob mark and is not a glob SHALL be an error.

#### Scenario: An unknown reader

- **WHEN** `formats` holds `toml: loud`
- **THEN** hakkiri reports `` `loud` is not a language hakkiri reads: use Markdown, MDX, Rust, TypeScript, TSX, JavaScript, Go, Swift, YAML, TOML, Shell or Text `` and exits 2

#### Scenario: A language with no reader

- **WHEN** `formats` holds `cfg: python`
- **THEN** hakkiri reports `` `python` is not a language hakkiri reads: use Markdown, MDX, Rust, TypeScript, TSX, JavaScript, Go, Swift, YAML, TOML, Shell or Text `` and exits 2

#### Scenario: A bad glob

- **WHEN** `formats` holds the key `a[`
- **THEN** hakkiri reports that it is not a glob and exits 2

### Requirement: on_unknown is a config key

`on_unknown` SHALL be a key of the project's config with the values of `--on-unknown`.  
It SHALL be resolved as the flag, then `HAKKIRI_ON_UNKNOWN`, then the config, then `text`, and an unknown word SHALL be an error.

#### Scenario: The order

- **WHEN** the config holds `on_unknown: error`, then `HAKKIRI_ON_UNKNOWN=ignore` is set, then `--on-unknown warn` is given
- **THEN** a selected file with no reader fails the run, then draws nothing, then draws a warning

#### Scenario: An unknown word

- **WHEN** the config holds `on_unknown: loud`
- **THEN** hakkiri reports an error that names `loud` and exits 2

### Requirement: A fault names the line of its own key

A fault about a key of the config SHALL name the line where the config writes that key.  
A fault about the value of a key SHALL name the line where the value starts.  
Both hold whatever characters the key holds and however the config is written.  
A fault about one item of a list SHALL name the line of that item.  
A glob key and a setting key SHALL each be one key, so `.`, `*`, `/`, `[` and `{` split nothing.

#### Scenario: Globs and settings that hold special characters

- **WHEN** `files` holds `'docs/**/*.md': {Team.Hint: yes}` on line 4, `'src/{a,b}/lib.rs'` on line 5 with `styles: [Other]` on line 6 and `Team.Hint: maybe` on line 7, and `'x[1].md': {styles: [Nope]}` on line 8
- **THEN** hakkiri names line 4 for the first fault, line 7 for the second and line 8 for the third

#### Scenario: A config written in flow style

- **WHEN** `files` is `{'docs/**/*.md': {Team.Hint: yes}, 'a.md': {styles: [Nope]}}` on line 4
- **THEN** both faults name line 4

#### Scenario: A later item of a list

- **WHEN** `styles` holds `Other` on line 6 and `Nope` on line 7, and `Nope` is no style
- **THEN** hakkiri names line 7, the line of the item, and not line 6

#### Scenario: An unknown vocabulary of a glob and a pattern that does not compile

- **WHEN** a glob's `vocab` holds `Team` and then `Nope`, or `token_ignores` holds `(z)` and then `(`, one item to a line
- **THEN** the fault names the line of `Nope`, or of `(`

#### Scenario: A glob or a setting with a block value

- **WHEN** a glob that is no glob has a block mapping below it, or a setting that names no rule has a block list below it
- **THEN** hakkiri names the line of the key, and not the line below it

#### Scenario: A refused key of a package's settings file

- **WHEN** a package's settings file writes `fail_level:` or a package `path:` with its value on the next line
- **THEN** hakkiri names the line of the key

### Requirement: A package style named Vale is skipped with a warning

hakkiri SHALL skip a package folder named for the built-in style `Vale`, as Vale does, and SHALL print one warning that names the folder and says the built-in style wins.  
The other styles of the package SHALL load as before.

#### Scenario: A package holds styles Acme and Vale

- **WHEN** a package holds `styles/Acme` and `styles/Vale`, and a glob lists `styles: [Acme, Vale]`
- **THEN** the rules of `Acme` run, no rule of the folder `Vale` runs, and `hakkiri check` prints one warning that `styles/Vale` is skipped because the built-in style `Vale` wins

### Requirement: A glob's specificity is four counts over its parts

A glob's specificity SHALL be four counts over the parts of its text between slashes, compared in order.  
They SHALL be the parts of literal characters only, then the parts that hold a literal character and something else, then the parts that are `**`, then the literal characters of the whole glob.  
More SHALL be more specific for each count but the third, where fewer SHALL be more specific.

#### Scenario: A folder beats a file type

- **WHEN** `files` holds `'docs/*': {Team.Filler: error}` and then `'*.mdx': {Team.Filler: warning}`
- **THEN** `docs/a.mdx` reports `Team.Filler` as an error, and `a.mdx` as a warning

#### Scenario: A deeper glob beats a shallower one written after it

- **WHEN** `files` holds `'docs/**/*.md': {Team.Filler: error}` and then `'*.md': {Team.Filler: warning}`
- **THEN** `docs/api/x.md` reports `Team.Filler` as an error, and `README.md` as a warning

### Requirement: A literal character matches only itself

A literal character SHALL be one outside a `[…]` class that is not `*` or `?`.  
A backslash and the character after it SHALL count as one literal character.

#### Scenario: More literal characters

- **WHEN** `files` holds `'a*.md': {Team.Filler: error}` and then `'*.md': {Team.Filler: warning}`
- **THEN** `a.md` reports `Team.Filler` as an error, and `b.md` as a warning

### Requirement: A glob with alternatives ranks as the alternative that matches

A glob with `{…}` groups SHALL be read as its alternatives, one for each choice in each group, each with its own specificity.  
For one file, the glob SHALL rank as the most specific of its alternatives that match that file.  
A glob with more than 256 alternatives SHALL be a fault that asks to split it into several globs.

#### Scenario: The alternative that matches

- **WHEN** `files` holds `'{*.md,docs/a.md}': {Team.Filler: error}` and then `'docs/*.md': {Team.Filler: warning}`
- **THEN** `a.md` and `docs/a.md` report `Team.Filler` as an error, and `docs/b.md` as a warning

#### Scenario: A skip list beats a package's file types

- **WHEN** a package sets `'*.{md,mdx,rs}': {styles: [Team]}` and the project sets `'{CHANGELOG.md,notes/**}': {styles: []}`
- **THEN** `a.md` runs `Team`, and `CHANGELOG.md` and `notes/b.md` run nothing

### Requirement: The settings cascade applies the globs in rank order

hakkiri SHALL rank every glob of the project and its packages once, by specificity, then by origin, then as written.  
It SHALL apply the globs that match a file from the lowest rank to the highest, a higher rank winning for the same key.  
A glob with alternatives SHALL take, for each file, the rank of its most specific alternative that matches that file.  
Only settings of the same key SHALL compete: the settings of every matching glob merge.

#### Scenario: Different keys merge

- **WHEN** a package sets `'*.md': {Team.Filler: error}` and the project sets `'docs/*.md': {code_span_size: 0}`
- **THEN** `docs/a.md` reports `Team.Filler` as an error and counts each code span as 0 words

### Requirement: Rule settings combine in rank order

A glob of higher rank SHALL win for the same rule, whether it turns the rule on or off or sets its level.  
Turning a rule on or off and setting its level SHALL compete as two keys, so a level stays when a glob of higher rank turns the rule off or on.  
The order of keys inside a glob SHALL NOT matter.

#### Scenario: A rule turned on after it was turned off

- **WHEN** `files` holds `'*.md': {styles: [Team], Team.Harsh: off}` and then `'a.md': {Team.Harsh: error}`
- **THEN** `a.md` runs `Team.Harsh` as an error, and `b.md` runs the rest of `Team`

#### Scenario: A more specific off written first

- **WHEN** `files` holds `'a.md': {Team.Filler: off}` and then `'*.md': {Team.Filler: warning}`
- **THEN** `a.md` runs nothing, and `b.md` runs `Team.Filler` as a warning

#### Scenario: A rule turned back on

- **WHEN** `files` holds `'*.md': {styles: [Team], Team.Harsh: off}` and then `'a.md': {Team.Harsh: on}`
- **THEN** `a.md` runs `Team.Harsh` as an error, the level of its rule file, and `b.md` runs the rest of `Team`

#### Scenario: A level that stays

- **WHEN** `files` holds `'*.md': {Team.Hint: error}`, then `'a?md': {Team.Hint: off}`, then `'a.md': {Team.Hint: on}`
- **THEN** `a.md` and `b.md` report `Team.Hint` as an error

#### Scenario: A level set again

- **WHEN** `files` holds `'*.md': {Team.Filler: warning}` and then `'a.md': {Team.Filler: error}`
- **THEN** `a.md` reports `Team.Filler` as an error, and `b.md` as a warning

#### Scenario: Keys in any order

- **WHEN** a glob holds `Team.Harsh: off` before `styles: [Team]`
- **THEN** the files it matches run `Team` without `Team.Harsh`

### Requirement: Style lists combine in rank order

A `styles` list SHALL replace the list of a lower rank and keep the rule settings of the lower ranks.  
An empty `styles` list SHALL drop every rule setting of a lower rank, and the rule settings of its own glob SHALL apply after it.

#### Scenario: A list of styles of a higher rank

- **WHEN** `files` holds `'*.md': {Team.Filler: warning}` and then `'a.md': {styles: [Other]}`
- **THEN** `a.md` runs `Other` and `Team.Filler`

#### Scenario: An empty list of styles

- **WHEN** `files` holds `'*.md': {styles: [Other], Team.Filler: warning}`, then `'a.md': {styles: []}`, then `'b.md': {styles: [], Team.Hint: warning}`
- **THEN** `a.md` runs nothing, `b.md` runs `Team.Hint` alone, and `c.md` runs `Other` and `Team.Filler`

### Requirement: Origin breaks a tie, and the top level ranks lowest

At equal specificity the project SHALL rank above a package and a later package above an earlier one, and within one file a later glob SHALL rank above an earlier one.  
The top level of each config file SHALL rank below every glob, in the same order of origin.

#### Scenario: A tie between a package and the project

- **WHEN** a package sets `'*.md': {Team.Filler: error}` and the project sets `'*.md': {Team.Filler: warning}`
- **THEN** `a.md` reports `Team.Filler` as a warning

#### Scenario: A package glob beats the project's top level

- **WHEN** a package sets `'*.md': {code_span_size: 0}` and the project sets the top-level `code_span_size: 2`
- **THEN** `README.md` counts each code span as 0 words

### Requirement: The project forces a value with !override

A value that the project's config tags `!override` SHALL apply after every value without the tag, whatever its rank, and the forced values SHALL apply among themselves in rank order.  
The tag SHALL be read on the value of a key directly under a glob and on the top-level `code_span_size`; a forced `params` mapping SHALL force every parameter it holds.  
A forced `styles` list SHALL be taken as written, and an empty forced list SHALL drop every rule setting without the tag.

#### Scenario: A forced level beats a more specific off

- **WHEN** a package sets `'docs/**/*.md': {Team.Filler: off}` and the project sets `'*.md': {Team.Filler: !override error}`
- **THEN** `docs/a.md` reports `Team.Filler` as an error

#### Scenario: A forced top-level value

- **WHEN** a package sets `'*.md': {code_span_size: 0}` and the project sets the top-level `code_span_size: !override 2`
- **THEN** `README.md` counts each code span as 2 words

### Requirement: Only the project forces, and only with !override

`!override` in a package's settings file, and any other tag on a value where hakkiri reads the tag, SHALL be an error that names the file and the line of its key.

#### Scenario: A package that forces

- **WHEN** a package's settings file holds `'*.md': {Team.Filler: !override off}` on line 4
- **THEN** hakkiri reports an error naming the package's file, line 4 and `!override`, and exits 2

#### Scenario: Another tag

- **WHEN** the project holds `'*.md': {Team.Filler: !important off}` on line 6
- **THEN** hakkiri reports an error naming line 6 and the tag `!important`, and exits 2

### Requirement: Lists add up over every matching glob

The lists of `vocab`, `block_ignores` and `token_ignores` SHALL add up over every glob that matches a file, in rank order.  
The forced lists SHALL add up among themselves and replace what the lists without the tag gave.

#### Scenario: Two ignore lists

- **WHEN** a package sets `'*.md': {token_ignores: ['\{\{[^}]*\}\}']}` and the project sets `'docs/*.md': {token_ignores: ['%[^%]*%']}`
- **THEN** `docs/a.md` hides both patterns from the prose rules, and `a.md` hides the first

#### Scenario: A forced list

- **WHEN** the same package glob holds, and the project sets `'docs/*.md': {token_ignores: !override ['%[^%]*%']}`
- **THEN** `docs/a.md` hides only `%…%`
