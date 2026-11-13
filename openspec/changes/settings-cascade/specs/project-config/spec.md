## ADDED Requirements

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

A literal character SHALL be one outside a `[…]` class and a `{…}` group that is not `*` or `?`.  
A backslash and the character after it SHALL count as one literal character.

#### Scenario: More literal characters

- **WHEN** `files` holds `'*.md': {Team.Filler: error}` and then `'*.{md,txt}': {Team.Filler: warning}`
- **THEN** `a.md` reports `Team.Filler` as an error, and `a.txt` as a warning

### Requirement: The settings cascade applies the globs in rank order

hakkiri SHALL rank every glob of the project and its packages once, by specificity, then by origin, then as written.  
It SHALL apply the globs that match a file from the lowest rank to the highest, a higher rank winning for the same key.  
Only settings of the same key SHALL compete: the settings of every matching glob merge.

#### Scenario: Different keys merge

- **WHEN** a package sets `'*.md': {Team.Filler: error}` and the project sets `'docs/*.md': {code_span_size: 0}`
- **THEN** `docs/a.md` reports `Team.Filler` as an error and counts each code span as 0 words

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

## MODIFIED Requirements

### Requirement: files maps globs to settings

hakkiri SHALL lint a file only when it matches a glob under `files`.  
It SHALL apply the settings of every glob the file matches in the rank of the settings cascade, a higher rank winning for the same key.

#### Scenario: Two globs match one file

- **WHEN** `files` holds `'*.{md,mdx,rs}': {styles: [ProseStyle]}` and then `'*.rs': {ProseStyle.BareURL: off}`
- **THEN** a Rust file runs every ProseStyle rule except `ProseStyle.BareURL`, and a Markdown file runs every ProseStyle rule

#### Scenario: A file no glob matches

- **WHEN** `files` holds only `'*.md'` and the project has `notes.txt`
- **THEN** hakkiri does not lint `notes.txt`

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

## REMOVED Requirements

### Requirement: Rule settings and style lists combine in the order written

**Reason**: The globs that match a file now apply in the rank of the settings cascade, not in the order written.  
**Migration**: The requirement "Rule settings and style lists combine in rank order" states the same rules over the rank.
