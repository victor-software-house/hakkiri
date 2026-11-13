## MODIFIED Requirements

### Requirement: A URL is a gap in the prose around the link

The text of an autolink or a bare URL that is its own destination SHALL be a gap in the nodes around it, and its `link` node SHALL hold no text.  
A link written with brackets whose text is its destination SHALL hold no text in its `link` node.  
A link whose destination differs from its text, such as a bare email address or a `www.` address, SHALL stay in the nodes around it.

#### Scenario: An autolink

- **WHEN** a Markdown file holds `Read <https://b.example> first.`
- **THEN** the `paragraph` node has the text `Read  first.`
- **AND** the file has no `link` node for it

#### Scenario: A bare URL in a comment

- **WHEN** a Rust file holds `// Read https://bare.example first.`
- **THEN** the `text.comment.line` node has the text `Read  first.`
- **AND** the file has no `link` node for it

#### Scenario: A link that shows its destination

- **WHEN** a Markdown file holds `See [https://a.example](https://a.example) now.`
- **THEN** the `paragraph` node has the text `See https://a.example now.`
- **AND** the file has no `link` node for it

#### Scenario: An email address and a www address

- **WHEN** a Markdown file holds `Mail me@b.example, <me@b.example> or www.b.example now.`
- **THEN** the `paragraph` node has the text `Mail me@b.example, me@b.example or www.b.example now.`
- **AND** the `link` nodes have the texts `me@b.example`, `me@b.example` and `www.b.example`

## ADDED Requirements

### Requirement: A bare ftp address is a gap

An `ftp://` address that starts a line or follows a space, `*`, `_`, `~` or `(` SHALL be a gap in the text around it, as an address with another scheme is.  
Its end SHALL be where Vale's converter ends it: closing punctuation, an unbalanced `)` and a character reference at its end are not part of it.

#### Scenario: An ftp address

- **WHEN** a Markdown file holds `Get ftp://files.example/a.txt. now.`
- **THEN** the `paragraph` node has the text `Get . now.`

### Requirement: A backslash escape is a gap

The backslash of an escape, and of a hard break written with a backslash, SHALL be a gap in the text, so the text holds the escaped character.

#### Scenario: An escape inside a word

- **WHEN** a Markdown file holds `A well\-known word.`
- **THEN** the `paragraph` node has the text `A well-known word.`

#### Scenario: A hard break with a backslash

- **WHEN** a Markdown file holds `First\` and `second.` on the next line
- **THEN** the `paragraph` node has the text `First` and `second.` on two lines, and no backslash

### Requirement: A table row has the cells of its header

A cell past the last cell of the header row SHALL be a gap, and a row with fewer cells SHALL hold the cells it has.

#### Scenario: A row with an extra cell

- **WHEN** a table has a header of two cells and a row `| a | b | c |`
- **THEN** the row holds the text of the cells `a` and `b`, and `c` is in no node

### Requirement: A footnote has the scope Vale gives it

A footnote reference SHALL be a gap in the text around it.  
A definition that a reference names SHALL be a `list` node of its content, where it stands.  
A definition that no reference names SHALL be a gap.

#### Scenario: A referenced and an unreferenced definition

- **WHEN** a Markdown file holds `A claim[^1] stands.`, `[^1]: The source.` and `[^2]: Unused.`
- **THEN** the `paragraph` node has the text `A claim stands.`
- **AND** one `list` node has the text `The source.`, and no node holds `Unused.`

### Requirement: An inline HTML element has the scope of its tag

An inline HTML element that has an opening tag and a closing tag among the same siblings SHALL be a node of the scope Vale gives its tag.  
The scope SHALL be `strong` for `b` and `strong`, `emphasis` for `i` and `em`, `code` for `code` and `tt`, and `link` for `a`.  
The two tags SHALL be gaps in the text of the node and of the paragraph around it.

#### Scenario: Bold and a link in HTML

- **WHEN** a Markdown file holds `Use <b>this</b> and <a href="u">that</a> now.`
- **THEN** the `paragraph` node has the text `Use this and that now.`
- **AND** a `strong` node has the text `this`, and a `link` node has the text `that`
