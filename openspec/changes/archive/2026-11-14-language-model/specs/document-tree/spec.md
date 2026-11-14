## ADDED Requirements

### Requirement: A fenced code block takes a language

A fenced code block SHALL take the langdex language that the first word of its info string names, up to a comma, so `rs` and `rust`, or `md` and `markdown`, name one language, and `rust,no_run` names Rust.  
The fence SHALL stay a gap in the prose around it.

#### Scenario: A rustdoc fence

- **WHEN** a Markdown file holds a fence opened with ```` ```rust,no_run ```` around `// A very good start.`, and a rule flags `very`
- **THEN** hakkiri reports `very` in the comment

#### Scenario: An alias of the language

- **WHEN** a Markdown file holds one fence opened with ```` ```rs ```` and one opened with ```` ```rust ````, each around `// A very good start.`, and a rule flags `very`
- **THEN** hakkiri reports `very` once in each fence

### Requirement: A fence gives the comments of its language

When hakkiri reads the comments of a fence's language, it SHALL parse the code of the fence with that language's grammar.  
Each comment SHALL be a node inside the node that holds the fence, read as the comments of a source file are.  
Each span of those nodes SHALL be the bytes of the Markdown or MDX file, and each such node, and every node inside it, SHALL have that language.  
A fence whose code does not map line for line onto the file SHALL give no nodes.

#### Scenario: A Rust fence

- **WHEN** a Markdown file holds `Intro.`, then a fence opened with ```` ```rust ```` around `// A very good start.` and `let x = 1;`, and a rule flags `very`
- **THEN** hakkiri reports `very` on the line of the comment, at its column in the file
- **AND** the comment node has the language Rust

#### Scenario: A fence in a list item

- **WHEN** a list item holds a fence indented by three spaces around `// A very good start.`
- **THEN** hakkiri reports `very` at its column in the file, past the indentation

#### Scenario: A scope for the fence's language

- **WHEN** a rule has `scope: text.comment.rs` and `tokens: [marker]`, and a Markdown file holds a ```` ```rust ```` fence around `// A marker.` and a ```` ```js ```` fence around `// A marker.`
- **THEN** hakkiri reports one alert, in the Rust fence

### Requirement: Other fences give no nodes

A fence of Markdown, MDX or Text, a fence with no info string, a fence of a language whose comments hakkiri does not read, and an indented code block SHALL give no nodes.

#### Scenario: A Markdown fence

- **WHEN** a fence opened with ```` ```md ```` holds `A very good start.`, and a rule flags `very`
- **THEN** hakkiri reports nothing for the fence

#### Scenario: A fence of a language with no comment reader

- **WHEN** a fence opened with ```` ```python ```` holds `# A very good start.`, and a rule flags `very`
- **THEN** hakkiri reports nothing for the fence
