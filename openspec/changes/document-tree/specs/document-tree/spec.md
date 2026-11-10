# document-tree

## Purpose

Every file hakkiri reads becomes one tree of nodes, each with a scope name and the byte span of its text in the file.  
Rules select text from this tree, and alerts point back into the file through the spans.

## ADDED Requirements

### Requirement: Every node holds a scope and a span

Each node in the tree SHALL hold its scope name, the byte span of its text in the file, its segments and its child nodes.  
The segments are the byte ranges of the node's text.  
The span runs from the start of the first segment to the end of the last.  

#### Scenario: A paragraph in a Markdown file

- **WHEN** a Markdown file holds `# Title`, a blank line, and `One line.`
- **THEN** the tree holds a `heading.h1` node spanning bytes 0 to 7 and a `paragraph` node spanning bytes 9 to 18

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

### Requirement: Markdown nodes carry Vale scope names

hakkiri SHALL give each Markdown node the scope the design's table names.  
The table covers paragraphs, headings, block quotes, list items, table rows and cells, links, code spans, strong and emphasis text, and image alt text.

#### Scenario: A nested list

- **WHEN** a Markdown file holds `- One` and, indented under it, `- Two`
- **THEN** the tree holds a `list` node for `One` whose children include a `list` node for `Two`

#### Scenario: A table

- **WHEN** a Markdown file holds a table with header `| Name | Note |` and one body row `| a | b |`
- **THEN** the tree holds two `table.row` nodes, the first with two `table.header` cells and the second with two `table.cell` cells, each span covering its cell text
- **AND** each row reads as its cells without the pipes, one space apart

### Requirement: Code and markup are not prose

Fenced code, raw HTML, link definitions and front matter SHALL carry no prose scope, and SHALL appear in the tree only under `raw`.  
Where such syntax sits inside a prose node, it SHALL be a gap between the segments of that node.

#### Scenario: A URL in a fenced block

- **WHEN** a Markdown file holds a fenced block with `curl https://example.com`
- **THEN** no prose node holds `https://example.com`, and the `raw` node of the file does

#### Scenario: HTML inside a paragraph

- **WHEN** a Markdown file holds `Click <a href="https://h.example">here</a>.`
- **THEN** the `paragraph` node has three segments, and its text is `Click here.`

### Requirement: MDX keeps prose and leaves code out

In an MDX file, text inside a JSX element SHALL be prose.  
Imports, exports, expressions, and the tags and attributes of JSX elements SHALL carry no prose scope.  
An export block SHALL end where its brackets close, also across a blank line.

A paragraph that holds an expression or a tag SHALL stay one node, with the expression and the tags as gaps between its segments.

#### Scenario: A JSX element with text

- **WHEN** an MDX file holds `import X from './x'` and `<Note title="t">Read this.</Note>`
- **THEN** the tree holds a `paragraph` node for `Read this.` and no prose node for the import or for `title="t"`

#### Scenario: An expression in a paragraph

- **WHEN** an MDX file holds `Text with {expression} inside.`
- **THEN** the tree holds one `paragraph` node whose text is `Text with  inside.`

### Requirement: Source files give the tree their comments only

For Rust, TypeScript and JavaScript files, hakkiri SHALL put only comment text in the tree, scoped `text.comment.line`, `text.comment.block`, `text.comment.doc.line` or `text.comment.doc.block` as the rule-files spec states.

#### Scenario: A Rust block comment

- **WHEN** a Rust file holds `/* A note. */` on its first line
- **THEN** the tree holds a `text.comment.block` node whose child `paragraph` spans bytes 3 to 10, the text `A note.`

### Requirement: String literals never reach the tree

No node in the tree of a source file SHALL hold text from a string literal, a raw string or a macro argument.

#### Scenario: A string that holds a URL

- **WHEN** a Rust file holds `let u = "https://example.com";` and `// see the docs`
- **THEN** the tree holds one comment node for `see the docs`, and no node holds `https://example.com`

### Requirement: Adjacent line comments join

Adjacent line comments of the same type SHALL join into one comment node, read as one Markdown text.

#### Scenario: A doc comment over three lines

- **WHEN** a Rust file holds three `///` lines, `/// First line`, `///` and `/// Second paragraph.`
- **THEN** the tree holds one `text.comment.doc.line` node with two `paragraph` children

### Requirement: Spans inside comments point into the file

A node inside a comment SHALL span the bytes of its text in the file, not in the text with markers stripped.  
A node SHALL hold one segment for each line of its text.  
A segment leaves out the comment marker and the indentation that hakkiri strips.

The text of the node is its segments in order, with a line feed between two segments on different lines.

#### Scenario: A link in a doc comment

- **WHEN** a Rust file holds the line `/// See [the guide](https://g.example).` indented by four spaces
- **THEN** the `link` node spans the bytes of `[the guide](https://g.example)` in that line of the file

#### Scenario: A paragraph over two line comments

- **WHEN** a Rust file holds the lines `// Read the guide` and `// before you start.`
- **THEN** the tree holds one `paragraph` node with two segments, the bytes of `Read the guide` and of `before you start.`, with no `//` marker in either
- **AND** its text is those two lines joined by a line feed

### Requirement: JSDoc tags are not prose

In a `/** */` comment of a TypeScript or JavaScript file, a tag name SHALL carry no prose scope.  
A type in braces after a tag SHALL carry none either, and neither SHALL the name after `@param`, `@typedef`, `@property`, `@prop`, `@arg`, `@argument`, `@template` or `@callback`.  
The description after them SHALL be prose, and the body of `@example` SHALL be code.  
A line inside a fenced block SHALL begin no tag.

#### Scenario: A parameter tag

- **WHEN** a TypeScript file holds `/** Sum two numbers.` on one line and `* @param a The first number. */` on the next
- **THEN** the tree holds prose for `Sum two numbers.` and `The first number.`, and none for `@param` or `a`

### Requirement: A banner leaves no punctuation

The run of `/` or `*` that follows the opener of a plain comment SHALL be stripped with the opener.

#### Scenario: A line of slashes

- **WHEN** a Rust file holds `////// Banner`
- **THEN** the tree holds a `paragraph` node whose text is `Banner`
