## ADDED Requirements

### Requirement: Doc line comments join across blank lines

hakkiri SHALL join Rust doc line comments of the same type across blank lines, where Vale 3.26.0 joins a line comment only when it follows the previous one directly.

#### Scenario: A fence across a blank line

- **WHEN** a Rust file holds `/// ```sh`, an empty line, `/// curl it` and `/// ```` `
- **THEN** hakkiri reads `curl it` as code, and Vale reads two comments, so a `text` rule sees `curl it` as prose ([doc-fence 1][doc-fence-1])

[doc-fence-1]: ../../design.md#1-doc-line-comments-join-across-blank-lines
