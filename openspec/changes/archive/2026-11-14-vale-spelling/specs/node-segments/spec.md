## MODIFIED Requirements

### Requirement: A URL is a gap in the prose around the link

The text of an autolink or a bare URL that is its own destination SHALL stay in its `link` node.  
It SHALL be a gap in the nodes around that node, since a URL counts no words in Vale.  
A link that is not its own destination, such as a bare email address or a `www.` address, SHALL stay in the nodes around it as well.

#### Scenario: An autolink

- **WHEN** a Markdown file holds `Read <https://b.example> first.`
- **THEN** the `paragraph` node has the text `Read  first.`
- **AND** the `link` node has the text `https://b.example`

#### Scenario: A bare URL in a comment

- **WHEN** a Rust file holds `// Read https://bare.example first.`
- **THEN** the `text.comment.line` node has the text `Read  first.`
- **AND** the `link` node has the text `https://bare.example`

#### Scenario: An email address and a www address

- **WHEN** a Markdown file holds `Mail me@b.example, <me@b.example> or www.b.example now.`
- **THEN** the `paragraph` node has the text `Mail me@b.example, me@b.example or www.b.example now.`
- **AND** the `link` nodes have the texts `me@b.example`, `me@b.example` and `www.b.example`
