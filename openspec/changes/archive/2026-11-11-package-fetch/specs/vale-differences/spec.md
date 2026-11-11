## MODIFIED Requirements

### Requirement: Configuration and packages are hakkiri's own

hakkiri SHALL read its settings from `.ctl/hakkiri.yaml` and take Vale styles only through `hakkiri import vale`, where Vale reads `.vale.ini`.

#### Scenario: A Vale configuration

- **WHEN** a project holds a `.vale.ini` and no `.ctl/hakkiri.yaml`
- **THEN** hakkiri reads no setting from it, and Vale reads it ([bootstrap 1][bootstrap-1])

#### Scenario: A rule is turned off

- **WHEN** a glob in the project config turns a rule off
- **THEN** it sets the rule to `off`, where Vale writes `NO` ([format 6][format-6])

#### Scenario: A package is fetched on use

- **WHEN** a project pins a package by `url` and `sha256`
- **THEN** hakkiri fetches it on the first run, checks the hash and keeps it in a cache folder outside the project, where Vale needs `vale sync`, accepts any zip and copies the styles into `StylesPath` ([package-fetch 1][package-fetch-1])

#### Scenario: A package that was changed

- **WHEN** a pinned package's download does not match its `sha256`
- **THEN** hakkiri refuses it and names the `url` and both hashes, where Vale installs a changed zip with no complaint ([package-fetch 3][package-fetch-3])

[package-fetch-1]: ../../design.md#1-vales-sync-is-the-reference-for-the-source
[package-fetch-3]: ../../design.md#3-the-checks-a-download-passes
[bootstrap-1]: ../../../2026-11-09-bootstrap-hakkiri/design.md#1-own-format-built-from-the-best-prior-art
[format-6]: ../../../2026-11-10-rule-format/design.md#6-the-project-config-uses-lowercase-keys
