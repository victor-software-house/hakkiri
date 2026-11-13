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

## REMOVED Requirements

### Requirement: Rule settings and style lists combine in the order written

**Reason**: The globs that match a file now apply in the rank of the settings cascade, not in the order written.  
**Migration**: The requirement "Rule settings and style lists combine in rank order" states the same rules over the rank.
