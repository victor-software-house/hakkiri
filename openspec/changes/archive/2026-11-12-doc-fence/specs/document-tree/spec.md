## MODIFIED Requirements

### Requirement: Adjacent line comments join

Adjacent line comments of the same type SHALL join into one comment node, read as one Markdown text.  
Doc line comments of the same type SHALL join across blank lines, as rustdoc reads them, and a line of code or a comment of another type SHALL end the run.

#### Scenario: A doc comment over three lines

- **WHEN** a Rust file holds three `///` lines, `/// First line`, `///` and `/// Second paragraph.`
- **THEN** the tree holds one `text.comment.doc.line` node with two `paragraph` children

#### Scenario: A fence across a blank line

- **WHEN** a Rust file holds `/// ```sh`, an empty line, `/// curl it` and `/// ```` `
- **THEN** the tree holds one `text.comment.doc.line` node, and `curl it` is in its code block

#### Scenario: A line of code ends the run

- **WHEN** a Rust file holds `/// ```sh`, `/// curl a`, `fn f() {}`, an empty line and `/// curl b`
- **THEN** the second `///` line is in a node of its own, read as prose

#### Scenario: Plain comments across a blank line

- **WHEN** a Rust file holds `// ```sh`, an empty line and `// curl c`
- **THEN** the two lines are in two nodes, and `curl c` is prose
