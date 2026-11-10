# node-segments

## Purpose

Every node of the document tree reads its text from segments: byte ranges of the file that hold prose and no syntax.  
A check reads the text, and an alert points at the span that covers it.

## ADDED Requirements

### Requirement: Every node holds segments

Each node SHALL hold its segments, the byte ranges of its text in the file, in order.  
The span of the node SHALL run from the start of the first segment to the end of the last.

#### Scenario: A heading

- **WHEN** a Markdown file holds `# Title`
- **THEN** the `heading.h1` node has one segment, the bytes of `Title`, and spans bytes 2 to 7

### Requirement: Segments join into the text of a node

The text of a node SHALL be its segments in order.  
A line feed SHALL separate two segments with a line break between them.  
One space SHALL separate two cells of a table row.  
Any other gap between segments SHALL read as nothing.

#### Scenario: A table row

- **WHEN** a Markdown file holds a table with the row `| a | b |`
- **THEN** the `table.row` node has the segments of its two cells, and its text is `a b`

### Requirement: A Markdown line is one segment

A Markdown paragraph, heading or inline node SHALL hold one segment for each line of its text.

A segment leaves out the line ending.  
It also leaves out the indentation and the block quote markers in front of the line.

#### Scenario: A paragraph in a block quote

- **WHEN** a Markdown file holds the lines `> First line` and `> second line.`
- **THEN** the tree holds one `paragraph` node with two segments, the bytes of `First line` and of `second line.`, with no `>` marker in either
- **AND** its text is those two lines joined by a line feed

#### Scenario: A list item that continues on an indented line

- **WHEN** a Markdown file holds the line `- Item first` and a line `item second` that is indented by two spaces
- **THEN** the `paragraph` node has two segments, the second without its two-space indent

#### Scenario: A file with Windows line endings

- **WHEN** a Markdown file holds `First line`, a carriage return and line feed, and `second line.`
- **THEN** the `paragraph` node has two segments, and its text holds no carriage return

### Requirement: Heading syntax is not text

An ATX heading SHALL leave out its opening run of `#` with the space after it, and its closing run of `#`.  
A setext heading SHALL leave out its underline.

#### Scenario: A closed heading

- **WHEN** a Markdown file holds `## Closed with more ###`
- **THEN** the `heading.h2` node has the text `Closed with more`

#### Scenario: A setext heading

- **WHEN** a Markdown file holds `Multi line`, `setext` and `======` on three lines
- **THEN** the `heading.h1` node has two segments and the text `Multi line` and `setext` joined by a line feed

### Requirement: Container text holds the content only

A block quote, a list item and a table row SHALL have the segments of the nodes inside them, in order.  
Their text SHALL hold no `>` marker, bullet, number or indentation.  
A block quote or list item with no prose inside it SHALL give no node.

#### Scenario: A nested quote

- **WHEN** a Markdown file holds `> First line` and, under it, `> > Nested one`
- **THEN** the outer `blockquote` node has the text `First line` and `Nested one` joined by a line feed

#### Scenario: A quote that holds only a definition

- **WHEN** a Markdown file holds `> [a]: https://a.example`
- **THEN** the tree holds no `blockquote` node

### Requirement: Syntax that is not prose is a gap

Fenced code, raw HTML, link definitions, front matter and MDX expressions SHALL be gaps between the segments of the prose node around them.  
The tags of a JSX element SHALL be gaps too, and its text SHALL be prose.  
A paragraph that holds one of them SHALL stay one node.

#### Scenario: HTML inside a paragraph

- **WHEN** a Markdown file holds `Click <a href="https://h.example">here</a>.`
- **THEN** the `paragraph` node has three segments, and its text is `Click here.`

#### Scenario: An expression in a paragraph

- **WHEN** an MDX file holds `Text with {expression} inside.`
- **THEN** the tree holds one `paragraph` node whose text is `Text with  inside.`

### Requirement: Segments inside comments point into the file

A node inside a comment SHALL hold one segment for each line of its text.  
A segment leaves out the comment marker and the indentation that hakkiri strips.

#### Scenario: A paragraph over two line comments

- **WHEN** a Rust file holds the lines `// Read the guide` and `// before you start.`
- **THEN** the tree holds one `paragraph` node with two segments, the bytes of `Read the guide` and of `before you start.`, with no `//` marker in either
- **AND** its text is those two lines joined by a line feed
