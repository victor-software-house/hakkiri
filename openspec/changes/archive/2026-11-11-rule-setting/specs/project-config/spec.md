## ADDED Requirements

### Requirement: A level or `on` runs a rule whichever style holds it

A rule name that gives a level SHALL run that rule on the matched files at that level, though no glob lists its style.  
`on` SHALL run the rule at the level of its rule file, and `off` SHALL stop a rule of a listed style and change nothing for a rule of a style no glob lists.

#### Scenario: A rule of a style no glob lists

- **WHEN** `files` holds `'a.md': {styles: [Other], Team.Filler: warning}`, the style `Team` also holds `Harsh`, and `a.md` holds `This is very terrible maybe bland.`
- **THEN** hakkiri reports `Team.Filler` as a warning on `very` and `Other.Bland` as a warning on `bland`, and runs no other rule of `Team`

#### Scenario: A rule setting and no styles

- **WHEN** `files` holds `'a.md': {Team.Hint: error}` and `a.md` holds `This is very terrible maybe bland.`
- **THEN** hakkiri reports one error, `Team.Hint` on `maybe`, though the rule's own level is `suggestion`, and exits 1

#### Scenario: A rule turned on at its own level

- **WHEN** `files` holds `'a.md': {styles: [Other], Team.Hint: on}`, and `a.md` holds `This is very terrible maybe bland.`
- **THEN** hakkiri reports `Team.Hint` as a suggestion, the level of its rule file, and `Other.Bland` as a warning, and exits 0

#### Scenario: A rule turned off in a style no glob lists

- **WHEN** `files` holds `'a.md': {styles: [Other], Team.Filler: off}`
- **THEN** hakkiri runs `Other` alone on `a.md`, with no error

#### Scenario: A style one glob lists and another glob enables by rule

- **WHEN** `files` holds `'*.md': {styles: [Other]}` and then `'a.md': {Team.Harsh: error}`
- **THEN** `a.md` runs `Other` and `Team.Harsh`, and `b.md` runs `Other` alone

### Requirement: Rule settings and style lists combine in the order written

A later glob SHALL win for the same rule, whether it turns the rule on or off or sets its level.  
A `styles` list SHALL replace the earlier list and keep the rule settings before it.  
An empty `styles` list SHALL drop every rule setting before it, and the rule settings of its own glob SHALL apply after it.  
The order of keys inside a glob SHALL NOT matter.

#### Scenario: A rule turned on after it was turned off

- **WHEN** `files` holds `'*.md': {styles: [Team], Team.Harsh: off}` and then `'a.md': {Team.Harsh: error}`
- **THEN** `a.md` runs `Team.Harsh` as an error, and `b.md` runs the rest of `Team`

#### Scenario: A rule turned off after it was enabled

- **WHEN** `files` holds `'*.md': {Team.Filler: warning}` and then `'a.md': {Team.Filler: off}`
- **THEN** `a.md` runs nothing, and `b.md` runs `Team.Filler` as a warning

#### Scenario: A rule turned back on

- **WHEN** `files` holds `'*.md': {styles: [Team], Team.Harsh: off}` and then `'a.md': {Team.Harsh: on}`
- **THEN** `a.md` runs `Team.Harsh` as an error, the level of its rule file, and `b.md` runs the rest of `Team`

#### Scenario: A level that stays

- **WHEN** `files` holds `'*.md': {Team.Hint: error}`, then `'a.md': {Team.Hint: off}`, then `'a?md': {Team.Hint: on}`
- **THEN** `a.md` and `b.md` report `Team.Hint` as an error

#### Scenario: A level set again

- **WHEN** `files` holds `'*.md': {Team.Filler: warning}` and then `'a.md': {Team.Filler: error}`
- **THEN** `a.md` reports `Team.Filler` as an error, and `b.md` as a warning

#### Scenario: A later list of styles

- **WHEN** `files` holds `'*.md': {Team.Filler: warning}` and then `'a.md': {styles: [Other]}`
- **THEN** `a.md` runs `Other` and `Team.Filler`

#### Scenario: An empty list of styles

- **WHEN** `files` holds `'*.md': {styles: [Other], Team.Filler: warning}`, then `'a.md': {styles: []}`, then `'b.md': {styles: []}` and `'b?md': {Team.Hint: warning}`
- **THEN** `a.md` runs nothing, `b.md` runs `Team.Hint` alone, and `c.md` runs `Other` and `Team.Filler`

#### Scenario: Keys in any order

- **WHEN** a glob holds `Team.Harsh: off` before `styles: [Team]`
- **THEN** the files it matches run `Team` without `Team.Harsh`

### Requirement: A rule setting does not make a clash an error

A rule setting SHALL NOT make hakkiri refuse a name clash or a bracket in the style of the rule it enables.  
When two files of a style that no glob lists give one name, and a setting enables that name, hakkiri SHALL run the later file in file name order.

#### Scenario: A clash in a style a setting enables

- **WHEN** `Team/Terms.yml` and `Team/Terms.custom.yml` both flag `very`, and `files` holds `'*.md': {styles: [Other], Team.Terms: warning}`
- **THEN** hakkiri accepts the config, and `This is very good.` draws one alert from `Team/Terms.yml`, the later file
