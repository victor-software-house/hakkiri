## ADDED Requirements

### Requirement: Origin breaks a tie, and the top level ranks lowest

At equal specificity the project SHALL rank above a package, and a later package above an earlier one.
Within one file, a later glob SHALL rank above an earlier one.
The top level of each config file SHALL rank below every glob, in the same order of origin.

#### Scenario: A tie between a package and the project

- **WHEN** a package sets `'*.md': {Team.Filler: error}` and the project sets `'*.md': {Team.Filler: warning}`
- **THEN** `a.md` reports `Team.Filler` as a warning

#### Scenario: A package glob beats the project's top level

- **WHEN** a package sets `'*.md': {code_span_size: 0}` and the project sets the top-level `code_span_size: 2`
- **THEN** `README.md` counts each code span as 0 words

### Requirement: The project forces a value with !override

A value that the project's config tags `!override` SHALL apply after every value without the tag, whatever its rank.
The forced values SHALL apply among themselves in rank order.

hakkiri SHALL read the tag on the value of a key directly under a glob, and on the top-level `code_span_size`.
A forced `params` mapping SHALL force every parameter it contains.
A forced `styles` list SHALL be taken as written, and an empty forced list SHALL drop every rule setting without the tag.

#### Scenario: A forced level beats a more specific off

- **WHEN** a package sets `'docs/**/*.md': {Team.Filler: off}` and the project sets `'*.md': {Team.Filler: !override error}`
- **THEN** `docs/a.md` reports `Team.Filler` as an error

#### Scenario: A forced top-level value

- **WHEN** a package sets `'*.md': {code_span_size: 0}` and the project sets the top-level `code_span_size: !override 2`
- **THEN** `README.md` counts each code span as 2 words

### Requirement: Only the project forces, and only with !override

`!override` in a package's settings file SHALL be an error.
Any other tag on a value where hakkiri reads the tag SHALL also be an error.
The error SHALL show the file and the line of the key.

#### Scenario: A package that forces

- **WHEN** a package's settings file contains `'*.md': {Team.Filler: !override off}` on line 4
- **THEN** hakkiri reports an error that shows the package's file, line 4 and `!override`, and exits 2

#### Scenario: Another tag

- **WHEN** the project contains `'*.md': {Team.Filler: !important off}` on line 6
- **THEN** hakkiri reports an error that shows line 6 and the tag `!important`, and exits 2

### Requirement: Lists add up over every matching glob

The lists of `vocab`, `block_ignores` and `token_ignores` SHALL add up over every glob that matches a file, in rank order.
The forced lists SHALL add up among themselves and replace what the lists without the tag gave.

#### Scenario: Two ignore lists

- **WHEN** a package sets `'*.md': {token_ignores: ['\{\{[^}]*\}\}']}` and the project sets `'docs/*.md': {token_ignores: ['%[^%]*%']}`
- **THEN** `docs/a.md` hides both patterns from the prose rules, and `a.md` hides the first

#### Scenario: A forced list

- **WHEN** the same package glob applies, and the project sets `'docs/*.md': {token_ignores: !override ['%[^%]*%']}`
- **THEN** `docs/a.md` hides only `%…%`

## MODIFIED Requirements

### Requirement: The project overrides a package

At equal specificity a setting in the project's config SHALL win over the same setting in a package's file.
A later package SHALL win over an earlier one.
A rule that a package sets `off` SHALL stay off until a setting of a glob that ranks higher turns it on.

The project's `level` SHALL win, and a package's `level` SHALL apply only when the project sets none.

#### Scenario: A project turns off a package rule

- **WHEN** a package sets `'*.md': {styles: [Team], Team.Filler: warning}`, and the project contains `files: {'a.md': {Team.Filler: off}}`
- **THEN** `a.md` draws no alert from `Team.Filler`, and `b.md` draws one

#### Scenario: A package rule that ships off

- **WHEN** a package sets `'*.md': {styles: [Team], Team.Filler: off}`, and `a.md` contains `This is very good.`
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
