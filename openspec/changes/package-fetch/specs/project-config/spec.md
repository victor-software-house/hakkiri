## MODIFIED Requirements

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

#### Scenario: A hash of the wrong shape

- **WHEN** an entry holds `sha256: abc123`
- **THEN** hakkiri reports ``line 2: packages[0]: `sha256` is 64 lowercase hex digits``, and exits with a failure status

#### Scenario: A url that is not http

- **WHEN** an entry holds `url: file:///tmp/Team.zip`
- **THEN** hakkiri reports ``line 2: packages[0]: a package `url` starts with `https://` or `http://` ``, and exits with a failure status

## ADDED Requirements

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
