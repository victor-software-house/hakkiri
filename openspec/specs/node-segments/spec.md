# node-segments Specification

## Purpose

Every node of the document tree reads its text from segments: byte ranges of the file that hold prose and no syntax.  
A check reads the text, and an alert points at the span that covers it.

## Requirements

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
A segment MAY carry a reading that replaces what the gap gives: nothing, one space or one line feed before the segment, and `*` for each character of the segment.  
The text of an HTML tag and the text around it SHALL read as the html-block-text spec defines.

#### Scenario: A table row

- **WHEN** a Markdown file holds a table with the row `| a | b |`
- **THEN** the `table.row` node has the segments of its two cells, and its text is `a b`

#### Scenario: A tag between words

- **WHEN** a Markdown file holds `a<br>marker<br/>b`
- **THEN** the `paragraph` node has three segments, and its text is `a marker b`

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

The text of an autolink or a bare URL that is its own destination SHALL stay in its `link` node, and SHALL be a gap in the nodes around it.  
A link written with brackets whose text is its destination SHALL keep that text in its `link` node and in the nodes around it.  
A link whose destination differs from its text, such as a bare email address or a `www.` address, SHALL stay in the nodes around it.

#### Scenario: An autolink

- **WHEN** a Markdown file holds `Read <https://b.example> first.`
- **THEN** the `paragraph` node has the text `Read  first.`
- **AND** a `link` node has the text `https://b.example`

#### Scenario: A bare URL in a comment

- **WHEN** a Rust file holds `// Read https://bare.example first.`
- **THEN** the `text.comment.line` node has the text `Read  first.`
- **AND** a `link` node has the text `https://bare.example`

#### Scenario: A link that shows its destination

- **WHEN** a Markdown file holds `See [https://a.example](https://a.example) now.`
- **THEN** the `paragraph` node has the text `See https://a.example now.`
- **AND** a `link` node has the text `https://a.example`

#### Scenario: An email address and a www address

- **WHEN** a Markdown file holds `Mail me@b.example, <me@b.example> or www.b.example now.`
- **THEN** the `paragraph` node has the text `Mail me@b.example, me@b.example or www.b.example now.`
- **AND** the `link` nodes have the texts `me@b.example`, `me@b.example` and `www.b.example`

### Requirement: A link that names its address is a gap in the prose around it

The text of a Markdown link that is one run of text equal to the address of the link SHALL stay in its `link` node.  
It SHALL be a gap in the nodes around that node, since Vale reads such a link as a URL and counts no words for it.  
The address SHALL be compared as goldmark writes it into the `href`: a byte outside its safe set percent-encoded, and a `%` before two hex digits kept.  
The address of a link by reference SHALL be the destination of the first definition of its label.

#### Scenario: A link that repeats its address

- **WHEN** a Markdown file holds ``See [a.md](a.md) now.``
- **THEN** the `paragraph` node has the text `See  now.`, and the `link` node has the text `a.md`

#### Scenario: A link with another address

- **WHEN** a Markdown file holds ``See [a.md](b.md) now.``
- **THEN** the `paragraph` node has the text `See a.md now.`

#### Scenario: A link by reference

- **WHEN** a Markdown file holds ``See [a.md][a] now.`` and the definition `[a]: a.md`
- **THEN** the `paragraph` node has the text `See  now.`

#### Scenario: A link whose text needs escaping in its address

- **WHEN** a Markdown file holds ``See [ä.md](ä.md) now.`` or ``See [a b](<a b>) now.``
- **THEN** the `paragraph` node keeps the text of the link, since goldmark writes the address as `%C3%A4.md` or `a%20b` and the text differs from it

#### Scenario: A link whose text is its escaped address

- **WHEN** a Markdown file holds ``See [a%20b](a%20b) now.``
- **THEN** the `paragraph` node has the text `See  now.`

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
