## ADDED Requirements

### Requirement: A top-level string of front matter is text

In a Markdown or MDX file, a rule scoped `text` SHALL read each top-level string value of front matter.  
The front matter is YAML between `---` lines, TOML between `+++` lines, or JSON between `;;;` lines.  
A key, a list, a nested mapping and a value that is not a string SHALL not be read.  
A TOML string below a table and a member of a nested JSON object SHALL not be read.

#### Scenario: Values of several kinds

- **WHEN** YAML front matter holds a plain, a quoted, a folded and a literal string, and a number, a boolean, a date, a list, a nested mapping and a key that holds `very`
- **THEN** a text rule that flags `very` reports the four strings, and no other

#### Scenario: Scalars that resolve to a string

- **WHEN** the values are `yes`, `no`, `on`, `off`, `y`, `n`, `12:30`, `1,000`, `1.2.3`, `v1.0`, `1e`, `0xZZ`, `a:b` and `!!str tagged`
- **THEN** each is read as text, and `null`, `~`, `true`, `0x1f`, `1_000`, `1e3`, `.inf`, `2024-01-01`, `<<` and `!!int 3` are not

#### Scenario: A scalar over lines and a comment

- **WHEN** a plain scalar goes on over two lines, and another holds `very # trail`
- **THEN** the first reads as its lines joined by a space, and the second as `very`

#### Scenario: TOML and JSON

- **WHEN** TOML front matter holds `title = "A very toml title"` and a table with a string, and JSON front matter holds a string, a list and a nested object
- **THEN** the title and the top-level JSON string are read, and the table, the list and the nested object are not

#### Scenario: An MDX file

- **WHEN** an MDX file starts with YAML front matter that holds a title
- **THEN** a text rule reads the title as it does in a Markdown file

### Requirement: A value has the scope of its key

A front matter value SHALL be read by a rule scoped `frontmatter`, `text.frontmatter`, `frontmatter.<key>` or `text.frontmatter.<key>`, and by no rule scoped `paragraph` or `sentence`.  
A scope name that goes on after `frontmatter` or `text.frontmatter` SHALL be a scope hakkiri defines.

#### Scenario: Scopes that read the title

- **WHEN** rules are scoped `frontmatter`, `frontmatter.title`, `text.frontmatter`, `text.frontmatter.title` and `text`, and a value `title` holds `very`
- **THEN** all five report it, and a rule scoped `frontmatter.title` reports no other key

#### Scenario: Scopes that do not

- **WHEN** a rule is scoped `paragraph` or `sentence`, and a `title` holds `very` or a sentence
- **THEN** it reports nothing in the front matter

### Requirement: Front matter is read as written

A block or token pattern of a glob SHALL not change the front matter, and a comment control SHALL not turn its rules off.  
An alert in a value SHALL lie at the text of the value in the file.

#### Scenario: A pattern that matches in a value

- **WHEN** a glob holds `token_ignores: ['{{.*?}}']`, with or without a group, and the title is `A {{ very }} title`
- **THEN** a text rule reports `very` in the title, and none in the same text of the body

#### Scenario: A folded scalar over two lines

- **WHEN** `description: >-` is followed by `Folded very` and `text here`
- **THEN** an alert for `here` lies on the second line

#### Scenario: A TOML value

- **WHEN** the TOML line reads `title = "A very toml title"` and a rule flags `title`
- **THEN** the alert lies at the word in the value, and not at the key
