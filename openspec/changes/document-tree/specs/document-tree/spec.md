# document-tree

## Purpose

Every file hakkiri reads becomes one tree of nodes, each with a scope name and the byte span of its text in the file.  
Rules select text from this tree, and alerts point back into the file through the spans.

## ADDED Requirements

### Requirement: Every node holds a scope and a span

Each node in the tree SHALL hold its scope name, the byte span of its text in the file, and its child nodes.

#### Scenario: A paragraph in a Markdown file

- **WHEN** a Markdown file holds `# Title`, a blank line, and `One line.`
- **THEN** the tree holds a `heading.h1` node spanning bytes 0 to 7 and a `paragraph` node spanning bytes 9 to 18

### Requirement: Markdown nodes carry Vale scope names

hakkiri SHALL give each Markdown node the scope the design's table names.  
The table covers paragraphs, headings, block quotes, list items, table rows and cells, links, code spans, strong and emphasis text, and image alt text.

#### Scenario: A nested list

- **WHEN** a Markdown file holds `- One` and, indented under it, `- Two`
- **THEN** the tree holds a `list` node for `One` whose children include a `list` node for `Two`

#### Scenario: A table

- **WHEN** a Markdown file holds a table with header `| Name | Note |` and one body row `| a | b |`
- **THEN** the tree holds two `table.row` nodes, the first with two `table.header` cells and the second with two `table.cell` cells, each span covering its cell text

### Requirement: Code and markup are not prose

Fenced code, raw HTML, link definitions and front matter SHALL carry no prose scope, and SHALL appear in the tree only under `raw`.

#### Scenario: A URL in a fenced block

- **WHEN** a Markdown file holds a fenced block with `curl https://example.com`
- **THEN** no prose node holds `https://example.com`, and the `raw` node of the file does

### Requirement: MDX keeps prose and leaves code out

In an MDX file, text inside a JSX element SHALL be prose.  
Imports, exports, expressions, and the tags and attributes of JSX elements SHALL carry no prose scope.

#### Scenario: A JSX element with text

- **WHEN** an MDX file holds `import X from './x'` and `<Note title="t">Read this.</Note>`
- **THEN** the tree holds a `paragraph` node for `Read this.` and no prose node for the import or for `title="t"`

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

#### Scenario: A link in a doc comment

- **WHEN** a Rust file holds the line `/// See [the guide](https://g.example).` indented by four spaces
- **THEN** the `link` node spans the bytes of `[the guide](https://g.example)` in that line of the file

### Requirement: JSDoc tags are not prose

In a `/** */` comment of a TypeScript or JavaScript file, a tag name SHALL carry no prose scope.  
The name after `@param` or `@typedef` SHALL carry no prose scope, and the description after it SHALL be prose.

#### Scenario: A parameter tag

- **WHEN** a TypeScript file holds `/** Sum two numbers.` on one line and `* @param a The first number. */` on the next
- **THEN** the tree holds prose for `Sum two numbers.` and `The first number.`, and none for `@param` or `a`
