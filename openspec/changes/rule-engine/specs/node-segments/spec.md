# Spec Delta

## ADDED Requirements

### Requirement: Inline markup is a gap

The markup of strong text, emphasis, links, autolinks, code spans and images SHALL be gaps between segments.  
These gaps SHALL cover the markers, the brackets, a link's target and title, an autolink's angle brackets and the backticks.

A link node SHALL hold the segments of its text, a code node those of its content, and an `alt` node those of its alt text.

#### Scenario: Strong text and a link

- **WHEN** a Markdown file holds `A **bold** [link](https://a.example) here.`
- **THEN** the `paragraph` node has the text `A bold link here.`
- **AND** the `link` node has the text `link`

#### Scenario: A code span and an image

- **WHEN** a Markdown file holds ``Run `cargo test` near ![the logo](l.png).``
- **THEN** the `code` node has the text `cargo test`, the `alt` node has the text `the logo`, and the `paragraph` node has the text `Run cargo test near .`

### Requirement: A URL is a gap in the prose around the link

The text of an autolink or a bare URL SHALL stay in its `link` node.  
It SHALL be a gap in the nodes around that node, since a URL counts no words in Vale.

#### Scenario: An autolink

- **WHEN** a Markdown file holds `Read <https://b.example> first.`
- **THEN** the `paragraph` node has the text `Read  first.`
- **AND** the `link` node has the text `https://b.example`

#### Scenario: A bare URL in a comment

- **WHEN** a Rust file holds `// Read https://bare.example first.`
- **THEN** the `text.comment.line` node has the text `Read  first.`
- **AND** the `link` node has the text `https://bare.example`
