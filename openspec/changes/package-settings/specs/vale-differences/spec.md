## ADDED Requirements

### Requirement: Package settings differ from Vale's package configuration

hakkiri SHALL read a package's settings as each scenario below says, where Vale 3.26.0 reads a package's `.vale.ini` otherwise.

#### Scenario: The file and its place

- **WHEN** a package holds `.ctl/hakkiri.yaml`
- **THEN** hakkiri reads it from the package folder, and Vale copies the package's `.vale.ini` to `StylesPath/.vale-config/<index>-<Name>.ini` at `vale sync` ([package settings 2][settings-2])

#### Scenario: A scalar in a package

- **WHEN** a package and the project both set `level`
- **THEN** the project's value applies, and Vale lets a package win for the keys it reads first, such as `WordTemplate` and the `[formats]` entries ([package settings 5][settings-5])

#### Scenario: An unknown key in a package

- **WHEN** a package's settings file holds a key hakkiri does not define
- **THEN** hakkiri refuses it with the line and the package, and Vale warns that the key is not a core option and ignores it ([package settings 7][settings-7])

#### Scenario: Packages that loop or nest deep

- **WHEN** a package lists itself through another package, or the chain is more than 10 packages deep
- **THEN** hakkiri refuses it, and Vale follows each `Packages` entry with no check ([package settings 3][settings-3])

#### Scenario: A nested package pinned by path

- **WHEN** a package's `packages` holds a `path` entry
- **THEN** hakkiri refuses it, and Vale installs a local folder named there ([package settings 3][settings-3])

#### Scenario: Ten or more packages

- **WHEN** a project lists ten or more packages
- **THEN** hakkiri applies them in the listed order, and Vale sorts their fragment files as text, so `10-` comes before `2-` ([package settings 6][settings-6])

#### Scenario: A project section that replaces package styles

- **WHEN** a project's section under another glob replaces the styles a package turned on
- **THEN** hakkiri prints nothing, and Vale warns once that the section replaces the styles of the package ([package settings 4][settings-4])

[settings-2]: ../../design.md#2-the-files-name-and-place-in-the-package
[settings-3]: ../../design.md#3-which-keys-a-package-may-set
[settings-4]: ../../design.md#4-a-styles-list-under-a-glob-the-package-also-writes
[settings-5]: ../../design.md#5-scalars-the-project-wins
[settings-6]: ../../design.md#6-order-across-packages
[settings-7]: ../../design.md#7-errors-name-the-package-file
