## ADDED Requirements

### Requirement: A package's settings file sits in its top folder

The top folder of a package SHALL be the folder that holds its `styles` folder, and a fetched zip's top folder SHALL be its one top folder.  
The top folder of a `path` entry that names a folder called `styles` SHALL be that folder's parent, except when the parent is the project root or holds it.  
A package with no top folder, and the project's own config, SHALL be no package's settings.

#### Scenario: A fetched package

- **WHEN** a package pinned by `url` and `sha256` holds `Pkg/.ctl/hakkiri.yaml` beside `Pkg/styles`
- **THEN** hakkiri reads it from the cache folder of the hash, with no further download

#### Scenario: Styles in the project root

- **WHEN** the project config at `.ctl/hakkiri.yaml` lists `path: styles`, a folder in the project root
- **THEN** hakkiri reads no settings from the root's `.ctl/hakkiri.yaml`, which is the project's own

#### Scenario: Another config in the root

- **WHEN** `--config cfg.yaml` names a config in the project root that lists `path: styles`, and the root holds a `.ctl/hakkiri.yaml` with an unknown key
- **THEN** hakkiri reads that `.ctl/hakkiri.yaml` as nothing, and reports no error for it

#### Scenario: A folder not called styles

- **WHEN** a config lists `path: pkg/rules` and `pkg` holds a `.ctl/hakkiri.yaml`
- **THEN** hakkiri reads no settings from it

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
