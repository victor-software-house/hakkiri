## ADDED Requirements

### Requirement: A fault names the line of its own key

A fault about a key of the config SHALL name the line where the config writes that key.  
A fault about the value of a key SHALL name the line where the value starts.  
Both hold whatever characters the key holds and however the config is written.  
A fault about one item of a list SHALL name the line of that item.  
A glob key and a setting key SHALL each be one key, so `.`, `*`, `/`, `[` and `{` split nothing.

#### Scenario: Globs and settings that hold special characters

- **WHEN** `files` holds `'docs/**/*.md': {Team.Hint: yes}` on line 4, `'src/{a,b}/lib.rs'` on line 5 with `styles: [Other]` on line 6 and `Team.Hint: maybe` on line 7, and `'x[1].md': {styles: [Nope]}` on line 8
- **THEN** hakkiri names line 4 for the first fault, line 7 for the second and line 8 for the third

#### Scenario: A config written in flow style

- **WHEN** `files` is `{'docs/**/*.md': {Team.Hint: yes}, 'a.md': {styles: [Nope]}}` on line 4
- **THEN** both faults name line 4

#### Scenario: A later item of a list

- **WHEN** `styles` holds `Other` on line 6 and `Nope` on line 7, and `Nope` is no style
- **THEN** hakkiri names line 7, the line of the item, and not line 6

#### Scenario: An unknown vocabulary of a glob and a pattern that does not compile

- **WHEN** a glob's `vocab` holds `Team` and then `Nope`, or `token_ignores` holds `(z)` and then `(`, one item to a line
- **THEN** the fault names the line of `Nope`, or of `(`

#### Scenario: A glob or a setting with a block value

- **WHEN** a glob that is no glob has a block mapping below it, or a setting that names no rule has a block list below it
- **THEN** hakkiri names the line of the key, and not the line below it

#### Scenario: A refused key of a package's settings file

- **WHEN** a package's settings file writes `fail_level:` or a package `path:` with its value on the next line
- **THEN** hakkiri names the line of the key
