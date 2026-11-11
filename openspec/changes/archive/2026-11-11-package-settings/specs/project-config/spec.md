## ADDED Requirements

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
