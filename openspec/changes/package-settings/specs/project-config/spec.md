## ADDED Requirements

### Requirement: A package may ship a settings file

hakkiri SHALL read `.ctl/hakkiri.yaml` inside each loaded package folder, the folder of styles, when it exists, in the format of the project config.  
It SHALL apply the settings of each package before the project's own, in the order of `packages`, as if they were written ahead of the project's `files` entries.  
A package with no such file SHALL change nothing.

#### Scenario: A package turns a style on

- **WHEN** the package folder `pkg` holds `Team/Filler.yml` flagging `very` and `.ctl/hakkiri.yaml` with `files: {'*.md': {styles: [Team]}}`, the project config is `packages: [{path: pkg}]` with no `files`, and `a.md` holds `This is very good.`
- **THEN** hakkiri reports one warning from `Team.Filler` in `a.md`

#### Scenario: A package with no settings file

- **WHEN** the same package holds no `.ctl/hakkiri.yaml`
- **THEN** hakkiri lints no file, as it does with a project that has no `files`

#### Scenario: A fetched package

- **WHEN** a package pinned by `url` and `sha256` holds `styles/.ctl/hakkiri.yaml` in its top folder
- **THEN** hakkiri reads it from the cache folder of the hash, with no further download

#### Scenario: A package folder that holds the project config

- **WHEN** the project config lists `path: .`, so the package folder holds the project's own `.ctl/hakkiri.yaml`
- **THEN** hakkiri reads that file once, as the project's

### Requirement: The project overrides a package

A setting in the project's config SHALL win over the same setting in a package's file, and a later package SHALL win over an earlier one.  
A rule that a package sets `off` SHALL stay off until a setting of the project or a later package turns it on.  
The project's `level` SHALL win, and a package's `level` SHALL apply only when the project sets none.

#### Scenario: A project turns off a package rule

- **WHEN** the package of the first scenario also sets `Team.Filler: warning`, and the project holds `files: {'a.md': {Team.Filler: off}}`
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

- **WHEN** the packages `a` and `b`, in that order, both set `Team.Filler` to a level
- **THEN** the level of `b` applies, and with the order reversed the level of `a` applies

### Requirement: A project's styles join those of a package under one glob

A project's `styles` list under the same glob text as a package's SHALL add to the package's list.  
A `styles` list under another glob SHALL replace the earlier list, as for any two globs of one config.

#### Scenario: The same glob

- **WHEN** a package sets `'*.md': {styles: [Team]}` and the project sets `'*.md': {styles: [Other]}`
- **THEN** a Markdown file runs `Team` and `Other`

#### Scenario: Another glob

- **WHEN** a package sets `'*.md': {styles: [Team]}` and the project sets `'*.{md,txt}': {styles: [Other]}`
- **THEN** a Markdown file runs `Other`, and the rule settings of the package still apply

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

### Requirement: A package may list packages

A package's `packages` SHALL be fetched like the project's, each entry holding a `url` and a `sha256`.  
A `path` entry in a package's file SHALL be an error.  
A chain of packages more than 10 deep SHALL be an error.  
A package named twice SHALL load once, by its `sha256` for a download and its folder for a `path`, and the packages a package lists SHALL come before it, in the order written.

#### Scenario: A nested package

- **WHEN** a package's `.ctl/hakkiri.yaml` holds `packages: [{url: https://x.example/base.zip, sha256: <hash>}]` and the zip holds `styles/Base/Filler.yml`
- **THEN** hakkiri fetches and caches `base.zip` as for a project entry, and a glob that lists `Base` runs `Base.Filler`

#### Scenario: A nested path

- **WHEN** a package's file holds `packages: [{path: ../base}]` on line 2
- **THEN** hakkiri reports an error naming `path`, line 2 and the package, and exits 2

#### Scenario: Too deep

- **WHEN** each of 11 packages lists the next, and the project lists the first
- **THEN** hakkiri reports an error that names the limit of 10, and exits 2

#### Scenario: One package named twice

- **WHEN** the project and a package both list the same `url` and `sha256`
- **THEN** hakkiri loads it once, with no error for a duplicate style

#### Scenario: Settings in order

- **WHEN** the package `top` lists `base`, both set `Team.Filler` to a level, and the project lists `top`
- **THEN** the level of `top` applies

### Requirement: Package vocabularies and the project's add up

The `vocab` names of the packages and the project SHALL add up, the packages' first.  
A name that no package holds, or that two packages hold, SHALL be an error on its line in the file that names it.

#### Scenario: Two lists

- **WHEN** a package sets `vocab: [Team]` and the project sets `vocab: [Own]`
- **THEN** hakkiri reads the vocabularies `Team` and `Own`

#### Scenario: A name no package holds

- **WHEN** a package's `.ctl/hakkiri.yaml` holds `vocab: [Nope]` on line 1 and no loaded package holds `config/vocabularies/Nope`
- **THEN** hakkiri reports an error naming `Nope`, line 1 and the package, and exits 2
