## MODIFIED Requirements

### Requirement: Code comments are scopes

In Rust, TypeScript, JavaScript, Go, Swift, YAML, TOML and shell files, hakkiri SHALL read `//` and `#` comments as `text.comment.line` and `/* */` comments as `text.comment.block`.  
It SHALL read Rust's `///` and `//!`, and Swift's `///`, as `text.comment.doc.line`, and `/** */`, Rust's `/*! */` and Swift's `/** */`, as `text.comment.doc.block`.

#### Scenario: A Rust block comment is read

- **WHEN** a rule has `scope: text.comment.block` and `tokens: [marker]`, and a Rust file holds `/* A marker here. */`
- **THEN** hakkiri reports one alert with the matched text `marker`

#### Scenario: A plain comment is not a doc comment

- **WHEN** a rule has `scope: text.comment.doc` and `tokens: [marker]`, and a Rust file holds `// A marker here.` and `/// A marker there.`
- **THEN** hakkiri reports one alert, on the `///` line

#### Scenario: A Swift doc comment

- **WHEN** a rule has `scope: text.comment.doc` and `tokens: [marker]`, and a Swift file holds `// A marker here.` and `/// A marker there.`
- **THEN** hakkiri reports one alert, on the `///` line

#### Scenario: A hash comment

- **WHEN** a rule has `scope: text.comment.line` and `tokens: [marker]`, and a YAML file holds `a: marker # A marker here.`
- **THEN** hakkiri reports one alert, in the comment
