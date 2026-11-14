## REMOVED Requirements

### Requirement: A file extension narrows a scope

**Reason**: A scope part names a language, not an extension.
**Migration**: Read "A language narrows a scope" below.

## ADDED Requirements

### Requirement: A language narrows a scope

A last scope part piece that names a language by any of its langdex names, aliases or extensions, such as `rs` in `raw.rs` or `rust` in `raw.rust`, SHALL limit the scope to nodes in that language.  
A negated part with a language, such as `~heading.rs`, SHALL leave out nodes only in that language.  
A scope item whose every part names another language SHALL select nothing.

#### Scenario: raw.rs in two files

- **WHEN** a rule has `scope: raw.rs` and `tokens: [marker]`, and a Markdown file holds `A marker.` and a Rust file holds `// A marker.`
- **THEN** hakkiri reports one alert, in the Rust file

#### Scenario: One language, several extensions

- **WHEN** a rule has `scope: text.comment.js` and `tokens: [marker]`, and the files `a.js`, `a.jsx`, `a.mjs` and `a.cjs` each hold `// A marker.`
- **THEN** hakkiri reports one alert in each file

#### Scenario: A name of the language

- **WHEN** a rule has `scope: text.comment.rust` and `tokens: [marker]`, and a Rust file holds `// A marker.`
- **THEN** hakkiri reports one alert

#### Scenario: A negated part for another language

- **WHEN** a rule has `scope: text & ~heading.rs` and `tokens: [marker]`, and a Markdown file holds `# A marker`
- **THEN** hakkiri reports one alert, on the heading

#### Scenario: An item that names only another language

- **WHEN** a rule has `scope: [~heading.rs]` and `tokens: [marker]`, and a Markdown file holds `# A marker` and `A marker.`
- **THEN** hakkiri reports no alert

### Requirement: A piece names a language only after a scope

A last piece SHALL name a language only when the whole part is not a scope hakkiri defines and the rest of it is one.  
A piece that names no language SHALL be an error.

#### Scenario: A scope name that ends like a language

- **WHEN** a rule has `scope: text.comment.line`
- **THEN** hakkiri reads it as the scope of line comments, in every language

#### Scenario: A piece that names no language

- **WHEN** a rule has `scope: text.nolang`
- **THEN** hakkiri refuses the rule with `` `text.nolang` is not a scope hakkiri defines ``
