## MODIFIED Requirements

### Requirement: A URL is a gap in the prose around the link

An autolink or a bare URL can be its own destination.
Its text SHALL stay in its `link` node, and SHALL be a gap in the nodes around it.

A link written with brackets whose text is its destination SHALL keep that text in its `link` node and in the nodes around it.
A link whose destination differs from its text SHALL stay in the nodes around it.
Such links include a bare email address and a `www.` address.

#### Scenario: An autolink

- **WHEN** a Markdown file contains `Read <https://b.example> first.`
- **THEN** the `paragraph` node has the text `Read  first.`
- **AND** a `link` node has the text `https://b.example`

#### Scenario: A bare URL in a comment

- **WHEN** a Rust file contains `// Read https://bare.example first.`
- **THEN** the `text.comment.line` node has the text `Read  first.`
- **AND** a `link` node has the text `https://bare.example`

#### Scenario: A link that shows its destination

- **WHEN** a Markdown file contains `See [https://a.example](https://a.example) now.`
- **THEN** the `paragraph` node has the text `See https://a.example now.`
- **AND** a `link` node has the text `https://a.example`

#### Scenario: An email address and a www address

- **WHEN** a Markdown file contains `Mail me@b.example, <me@b.example> or www.b.example now.`
- **THEN** the `paragraph` node has the text `Mail me@b.example, me@b.example or www.b.example now.`
- **AND** the `link` nodes have the texts `me@b.example`, `me@b.example` and `www.b.example`

## ADDED Requirements

### Requirement: A bare ftp address is a gap

An `ftp://` address can start a line or follow a space, `*`, `_`, `~` or `(`.
Such an address SHALL be a gap in the text around it, as an address with another scheme is.

Its end SHALL be where Vale's converter ends it.
Closing punctuation, an unbalanced `)` and a character reference at its end are not part of it.

#### Scenario: An ftp address

- **WHEN** a Markdown file contains `Get ftp://files.example/a.txt. now.`
- **THEN** the `paragraph` node has the text `Get . now.`

### Requirement: A backslash escape is a gap

The backslash of an escape, and of a hard break written with a backslash, SHALL be a gap in the text.
So the text contains the escaped character.

#### Scenario: An escape inside a word

- **WHEN** a Markdown file contains `A well\-known word.`
- **THEN** the `paragraph` node has the text `A well-known word.`

#### Scenario: A hard break with a backslash

- **WHEN** a Markdown file contains `First\` and `second.` on the next line
- **THEN** the `paragraph` node has the text `First` and `second.` on two lines, and no backslash

### Requirement: A table row has the cells of its header

A cell past the last cell of the header row SHALL be a gap.
A row with fewer cells SHALL keep the cells it has.

#### Scenario: A row with an extra cell

- **WHEN** a table has a header of two cells and a row `| a | b | c |`
- **THEN** the row contains the text of the cells `a` and `b`, and `c` is in no node

### Requirement: A footnote has the scope Vale gives it

A footnote reference SHALL be a gap in the text around it.
When a reference points to a definition, the definition SHALL be a `list` node of its content, where it stands.
A definition that no reference points to SHALL be a gap.

#### Scenario: A referenced and an unreferenced definition

- **WHEN** a Markdown file contains `A claim[^1] stands.`, `[^1]: The source.` and `[^2]: Unused.`
- **THEN** the `paragraph` node has the text `A claim stands.`
- **AND** one `list` node has the text `The source.`, and no node contains `Unused.`

### Requirement: An inline HTML element has the scope of its tag

An inline HTML element can have an opening tag and a closing tag among the same siblings.
Such an element SHALL be a node of the scope Vale gives its tag.

The scope SHALL be `strong` for `b` and `strong`, `emphasis` for `i` and `em`, `code` for `code` and `tt`, and `link` for `a`.
The two tags SHALL be gaps in the text of the node and of the paragraph around it.

#### Scenario: Bold and a link in HTML

- **WHEN** a Markdown file contains `Use <b>this</b> and <a href="u">that</a> now.`
- **THEN** the `paragraph` node has the text `Use this and that now.`
- **AND** a `strong` node has the text `this`, and a `link` node has the text `that`
