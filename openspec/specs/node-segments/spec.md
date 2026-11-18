# node-segments Specification

## Purpose

Every node of the document tree reads its text from segments: byte ranges of the file that contain prose and no syntax.  
A check reads the text, and an alert points at the span that covers it.

## Requirements

### Requirement: Every node contains segments

Each node SHALL contain its segments, the byte ranges of its text in the file, in order.  
The span of the node SHALL run from the start of the first segment to the end of the last.

#### Scenario: A heading

- **WHEN** a Markdown file contains `# Title`
- **THEN** the `heading.h1` node has one segment, the bytes of `Title`, and spans bytes 2 to 7

### Requirement: Segments join into the text of a node

The text of a node SHALL be its segments in order.  
A line feed SHALL separate two segments with a line break between them.  
One space SHALL separate two cells of a table row.  
Any other gap between segments SHALL read as nothing.

A segment MAY carry a reading that replaces what the gap gives.  
The reading is nothing, one space or one line feed before the segment, and `*` for each character of the segment.  
The text of an HTML tag and the text around it SHALL read as the html-block-text spec defines.

#### Scenario: A table row

- **WHEN** a Markdown file contains a table with the row `| a | b |`
- **THEN** the `table.row` node has the segments of its two cells, and its text is `a b`

#### Scenario: A tag between words

- **WHEN** a Markdown file contains `a<br>marker<br/>b`
- **THEN** the `paragraph` node has three segments, and its text is `a marker b`

### Requirement: A Markdown line is one segment

A Markdown paragraph, heading or inline node SHALL have one segment for each line of its text.

A segment leaves out the line ending.  
It also leaves out the indentation and the block quote markers in front of the line.

#### Scenario: A paragraph in a block quote

- **WHEN** a Markdown file contains the lines `> First line` and `> second line.`
- **THEN** the tree contains one `paragraph` node with two segments, the bytes of `First line` and of `second line.`, with no `>` marker in either
- **AND** its text is those two lines joined by a line feed

#### Scenario: A list item that continues on an indented line

- **WHEN** a Markdown file contains the line `- Item first` and a line `item second` that is indented by two spaces
- **THEN** the `paragraph` node has two segments, the second without its two-space indent

#### Scenario: A file with Windows line endings

- **WHEN** a Markdown file contains `First line`, a carriage return and line feed, and `second line.`
- **THEN** the `paragraph` node has two segments, and its text contains no carriage return

### Requirement: Heading syntax is not text

An ATX (hash-prefixed) heading SHALL leave out its opening run of `#` with the space after it, and its closing run of `#`.  
A setext heading SHALL leave out its underline.

#### Scenario: A closed heading

- **WHEN** a Markdown file contains `## Closed with more ###`
- **THEN** the `heading.h2` node has the text `Closed with more`

#### Scenario: A setext heading

- **WHEN** a Markdown file contains `Multi line`, `setext` and `======` on three lines
- **THEN** the `heading.h1` node has two segments and the text `Multi line` and `setext` joined by a line feed

### Requirement: Container text contains the content only

A block quote, a list item and a table row SHALL have the segments of the nodes inside them, in order.  
Their text SHALL contain no `>` marker, bullet, number or indentation.  
A block quote or list item with no prose inside it SHALL give no node.

#### Scenario: A nested quote

- **WHEN** a Markdown file contains `> First line` and, under it, `> > Nested one`
- **THEN** the outer `blockquote` node has the text `First line` and `Nested one` joined by a line feed

#### Scenario: A quote that contains only a definition

- **WHEN** a Markdown file contains `> [a]: https://a.example`
- **THEN** the tree contains no `blockquote` node

### Requirement: Syntax that is not prose is a gap

Fenced code, raw HTML, link definitions, front matter and Markdown with components (MDX) expressions SHALL be gaps in the prose node around them.  
The tags of a JSX (JavaScript XML) element SHALL be gaps too, and its text SHALL be prose.  
A paragraph that contains one of them SHALL stay one node.

#### Scenario: HTML inside a paragraph

- **WHEN** a Markdown file contains `Click <a href="https://h.example">here</a>.`
- **THEN** the `paragraph` node has three segments, and its text is `Click here.`

#### Scenario: An expression in a paragraph

- **WHEN** an MDX file contains `Text with {expression} inside.`
- **THEN** the tree contains one `paragraph` node whose text is `Text with  inside.`

### Requirement: Segments inside comments point into the file

A node inside a comment SHALL have one segment for each line of its text.  
A segment leaves out the comment marker and the indentation that hakkiri strips.

#### Scenario: A paragraph over two line comments

- **WHEN** a Rust file contains the lines `// Read the guide` and `// before you start.`
- **THEN** the tree contains one `paragraph` node with two segments, the bytes of `Read the guide` and of `before you start.`, with no `//` marker in either
- **AND** its text is those two lines joined by a line feed

### Requirement: Inline markup is a gap

The markup of strong text, emphasis, links, autolinks, code spans and images SHALL be gaps between segments.  
These gaps SHALL cover the markers, the brackets, a link's target and title, an autolink's angle brackets and the backticks.

A link node SHALL have the segments of its text, a code node those of its content, and an `alt` node those of its alt text.

#### Scenario: Strong text and a link

- **WHEN** a Markdown file contains `A **bold** [link](https://a.example) here.`
- **THEN** the `paragraph` node has the text `A bold link here.`
- **AND** the `link` node has the text `link`

#### Scenario: A code span and an image

- **WHEN** a Markdown file contains ``Run `cargo test` near ![the logo](l.png).``
- **THEN** the `code` node has the text `cargo test`, the `alt` node has the text `the logo`, and the `paragraph` node has the text `Run cargo test near .`

### Requirement: A URL is a gap in the prose around the link

The text of an autolink or a bare URL that is its own destination SHALL stay in its `link` node.  
It SHALL be a gap in the nodes around it.

The requirement "A link that names its address is a gap in the prose around it" covers a bracketed link whose text is its address.  
A link whose destination differs from its text, such as a bare email address or a `www.` address, SHALL stay in the nodes around it.

#### Scenario: An autolink

- **WHEN** a Markdown file contains `Read <https://b.example> first.`
- **THEN** the `paragraph` node has the text `Read  first.`
- **AND** a `link` node has the text `https://b.example`

#### Scenario: A bare URL in a comment

- **WHEN** a Rust file contains `// Read https://bare.example first.`
- **THEN** the `text.comment.line` node has the text `Read  first.`
- **AND** a `link` node has the text `https://bare.example`

#### Scenario: An email address and a www address

- **WHEN** a Markdown file contains `Mail me@b.example, <me@b.example> or www.b.example now.`
- **THEN** the `paragraph` node has the text `Mail me@b.example, me@b.example or www.b.example now.`
- **AND** the `link` nodes have the texts `me@b.example`, `me@b.example` and `www.b.example`

### Requirement: A link that names its address is a gap in the prose around it

The text of a Markdown link that is one run of text equal to the address of the link SHALL stay in its `link` node.  
It SHALL be a gap in the nodes around that node.  
Vale reads such a link as a URL and counts no words for it.

The address SHALL be compared as goldmark writes it into the `href`.  
goldmark percent-encodes a byte outside its safe set, and keeps a `%` before two hex digits.  
The address of a link by reference SHALL be the destination of the first definition of its label.

#### Scenario: A link that repeats its address

- **WHEN** a Markdown file contains ``See [a.md](a.md) now.``
- **THEN** the `paragraph` node has the text `See  now.`, and the `link` node has the text `a.md`

#### Scenario: A link with another address

- **WHEN** a Markdown file contains ``See [a.md](b.md) now.``
- **THEN** the `paragraph` node has the text `See a.md now.`

#### Scenario: A link by reference

- **WHEN** a Markdown file contains ``See [a.md][a] now.`` and the definition `[a]: a.md`
- **THEN** the `paragraph` node has the text `See  now.`

#### Scenario: A link whose text needs escaping in its address

- **WHEN** a Markdown file contains ``See [ä.md](ä.md) now.`` or ``See [a b](<a b>) now.``
- **THEN** the `paragraph` node keeps the text of the link, since goldmark writes the address as `%C3%A4.md` or `a%20b` and the text differs from it

#### Scenario: A link whose text is its escaped address

- **WHEN** a Markdown file contains ``See [a%20b](a%20b) now.``
- **THEN** the `paragraph` node has the text `See  now.`

### Requirement: A bare ftp address is a gap

An `ftp://` address that starts a line or follows a space, `*`, `_`, `~` or `(` SHALL be a gap in the text around it.  
An address with another scheme is a gap the same way.  
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
A row with fewer cells SHALL contain the cells it has.

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

An inline HTML element with an opening tag and a closing tag among the same siblings SHALL be a node.  
Its scope SHALL be the one Vale gives its tag.  
The scope SHALL be `strong` for `b` and `strong`, `emphasis` for `i` and `em`, `code` for `code` and `tt`, and `link` for `a`.

The two tags SHALL be gaps in the text of the node and of the paragraph around it.

#### Scenario: Bold and a link in HTML

- **WHEN** a Markdown file contains `Use <b>this</b> and <a href="u">that</a> now.`
- **THEN** the `paragraph` node has the text `Use this and that now.`
- **AND** a `strong` node has the text `this`, and a `link` node has the text `that`
