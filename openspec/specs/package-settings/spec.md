# package-settings Specification

## Purpose

A package ships a settings file in its top folder, in the format of the project config.  
hakkiri applies it under the project's own.  
The file contains the packages a package lists and the vocabularies it uses.

## Requirements

### Requirement: A package's settings file is in its top folder

The top folder of a package SHALL be the folder that contains its `styles` folder.  
A fetched zip's top folder SHALL be its one top folder.

A `path` entry may point to a folder called `styles`.  
Its top folder SHALL then be that folder's parent, except when the parent is the project root or contains it.  
A package with no top folder, and the project's own config, SHALL be no package's settings.

#### Scenario: A fetched package

- **WHEN** a package pinned by `url` and `sha256` contains `Pkg/.ctl/hakkiri.yaml` beside `Pkg/styles`
- **THEN** hakkiri reads it from the cache folder of the hash, with no further download

#### Scenario: Styles in the project root

- **WHEN** the project config at `.ctl/hakkiri.yaml` lists `path: styles`, a folder in the project root
- **THEN** hakkiri reads no settings from the root's `.ctl/hakkiri.yaml`, which is the project's own

#### Scenario: Another config in the root

- **WHEN** `--config cfg.yaml` points to a config in the project root that lists `path: styles`, and the root contains a `.ctl/hakkiri.yaml` with an unknown key
- **THEN** hakkiri reads that `.ctl/hakkiri.yaml` as nothing, and reports no error for it

#### Scenario: A folder not called styles

- **WHEN** a config lists `path: pkg/rules` and `pkg` contains a `.ctl/hakkiri.yaml`
- **THEN** hakkiri reads no settings from it

### Requirement: A package may list packages

A package's `packages` SHALL be fetched like the project's, each entry containing a `url` and a `sha256`.  
In a package's file, a `path` entry SHALL be an error.  
So SHALL a chain of packages more than 10 deep.

A package listed twice SHALL load once, by its `sha256` for a download and its folder for a `path`.  
The packages a package lists SHALL come before it, in the order written.

#### Scenario: A nested package

- **WHEN** a package's `.ctl/hakkiri.yaml` contains `packages: [{url: https://x.example/base.zip, sha256: <hash>}]` and the zip contains `styles/Base/Filler.yml`
- **THEN** hakkiri fetches and caches `base.zip` as for a project entry, and a glob that lists `Base` runs `Base.Filler`

#### Scenario: A nested path

- **WHEN** a package's file contains `packages: [{path: ../base}]` on line 2
- **THEN** hakkiri reports an error that shows `path`, line 2 and the package, and exits 2

#### Scenario: Too deep

- **WHEN** each of 11 packages lists the next, and the project lists the first
- **THEN** hakkiri reports an error that shows the limit of 10, and exits 2

#### Scenario: One package listed twice

- **WHEN** the project and a package both list the same `url` and `sha256`
- **THEN** hakkiri loads it once, with no error for a duplicate style

#### Scenario: Settings in order

- **WHEN** the package `top` lists `base`, both set `Team.Filler` to a level, and the project lists `top`
- **THEN** the level of `top` applies

### Requirement: Package vocabularies and the project's add up

The `vocab` lists of the packages and the project SHALL add up, the packages' first.  
An empty `vocab` list SHALL drop the names before it, as Vale's `Vocab = NONE` does, and a missing key SHALL keep them.

The same SHALL apply between two packages, where the later one wins.  
So SHALL the `vocab` of a glob, which drops only the lists of earlier configs for the same glob text.  
A name that no package or two packages contain SHALL be an error on its line in the file that lists it.

#### Scenario: Two lists

- **WHEN** a package sets `vocab: [Team]` and the project sets `vocab: [Own]`
- **THEN** hakkiri reads the vocabularies `Team` and `Own`

#### Scenario: An empty list

- **WHEN** a package sets `vocab: [Team]` and the project sets `vocab: []`
- **THEN** hakkiri reads no vocabulary listed by a package

#### Scenario: A later package

- **WHEN** the package `a` sets `vocab: [Team]` and the later package `b` sets `vocab: []`
- **THEN** hakkiri reads no vocabulary listed by a package

#### Scenario: A glob's empty list

- **WHEN** a package's globs `'*.md'` and `'a.md'` name `Docs` and `Own`, and the project's glob `'*.md'` sets `vocab: []`
- **THEN** `a.md` and `b.md` take no vocabulary from the package's `'*.md'`, and `a.md` still takes `Own`

#### Scenario: Another glob text

- **WHEN** the project's glob is `'[ab].md'` with `vocab: []`, and the package's globs are those of the last scenario
- **THEN** `a.md` and `b.md` still take `Docs`, and `a.md` still takes `Own`

#### Scenario: A missing key

- **WHEN** a package sets `vocab: [Team]` and the project sets no `vocab`
- **THEN** hakkiri reads the vocabulary `Team`

#### Scenario: A name no package contains

- **WHEN** a package's `.ctl/hakkiri.yaml` contains `vocab: [Nope]` on line 1 and no loaded package contains `config/vocabularies/Nope`
- **THEN** hakkiri reports an error that shows `Nope`, line 1 and the package, and exits 2

### Requirement: A package's settings do not set the run options

A package's settings file SHALL NOT contain `fail_level`, `gitignore` or `on_unmatched`.  
A file that does SHALL be an error that gives the key, its line and the package.

#### Scenario: A package sets all three

- **WHEN** a package's `.ctl/hakkiri.yaml` contains `gitignore: false` on line 2, `fail_level: warning` on line 3 and `on_unmatched: error` on line 4
- **THEN** hakkiri reports an error for each, such as `pkg/.ctl/hakkiri.yaml: line 2: gitignore: belongs to the project's config`, and exits 2

### Requirement: A package's settings contain formats and not on_unknown

A package's settings file MAY contain `formats`, and the entries of the packages and the project SHALL add up.  
A later entry SHALL replace an earlier one with the same key.  
It SHALL NOT contain `on_unknown`, and a file that does SHALL be an error that shows the key, its line and the package.

#### Scenario: The project overrides the package

- **WHEN** a package maps `txt: md` and the project maps `txt: text`
- **THEN** `a.txt` is read as plain text, and read as Markdown when the project has no entry

#### Scenario: A package sets on_unknown

- **WHEN** a package's file contains `on_unknown: warn` on line 5
- **THEN** hakkiri reports `pkg/.ctl/hakkiri.yaml: line 5: on_unknown: belongs to the project's config` and exits 2
