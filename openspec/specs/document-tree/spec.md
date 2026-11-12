# document-tree Specification

## Purpose

Every file hakkiri reads becomes one tree of nodes, each with a scope name and the byte span of its text in the file.  
Rules select text from this tree, and alerts point back into the file through the spans.

## Requirements

### Requirement: Every node holds a scope and a span

Each node in the tree SHALL hold its scope name, the byte span of its text in the file, and its child nodes.  
The node-segments spec states which ranges make the text of a node and how the span covers them.

#### Scenario: A paragraph in a Markdown file

- **WHEN** a Markdown file holds `# Title`, a blank line, and `One line.`
- **THEN** the tree holds a `heading.h1` node spanning bytes 2 to 7 and a `paragraph` node spanning bytes 9 to 18

### Requirement: Markdown nodes carry Vale scope names

hakkiri SHALL give each Markdown node the scope the [design's table][tree-4] names.  
The table covers paragraphs, headings, block quotes, list items, table rows and cells, links, code spans, strong and emphasis text, and image alt text.  
The text of an HTML block adds table and figure captions, the content of a `data` element as `meta`, and the description of a page.

#### Scenario: A nested list

- **WHEN** a Markdown file holds `- One` and, indented under it, `- Two`
- **THEN** the tree holds a `list` node for `One` whose children include a `list` node for `Two`

#### Scenario: A table

- **WHEN** a Markdown file holds a table with header `| Name | Note |` and one body row `| a | b |`
- **THEN** the tree holds two `table.row` nodes, the first with two `table.header` cells and the second with two `table.cell` cells, each span covering its cell text

### Requirement: Code and markup are not prose

Fenced code, the tags of raw HTML, link definitions and front matter SHALL carry no prose scope, and SHALL appear in the tree only under `raw`.  
Inside a prose node, the node-segments spec makes such syntax a gap in the text.  
The text inside an HTML block is prose, and the html-block-text spec states which text and under which scope.

#### Scenario: A URL in a fenced block

- **WHEN** a Markdown file holds a fenced block with `curl https://example.com`
- **THEN** no prose node holds `https://example.com`, and the `raw` node of the file does

#### Scenario: A tag and its text in an HTML block

- **WHEN** a Markdown file holds an HTML block `<div class="note">Read this.</div>`
- **THEN** the tree holds a `paragraph` node for `Read this.` and no prose node for `<div class="note">`

### Requirement: MDX keeps prose and leaves code out

In an MDX file, text inside a JSX element SHALL be prose.  
Imports, exports, expressions, and the tags and attributes of JSX elements SHALL carry no prose scope.

#### Scenario: A JSX element with text

- **WHEN** an MDX file holds `import X from './x'` and `<Note title="t">Read this.</Note>`
- **THEN** the tree holds a `paragraph` node for `Read this.` and no prose node for the import or for `title="t"`

### Requirement: Export blocks end at the end of their code

An export block SHALL end at the first blank line after which its code parses as TSX.  
Incomplete code SHALL read on past the blank line.

#### Scenario: A quote or a regular expression in an export block

- **WHEN** an MDX file holds `export const re = /'/`, a blank line and a paragraph
- **THEN** the tree holds a `paragraph` node for that paragraph and none for the export

#### Scenario: Incomplete code reads on

- **WHEN** an MDX file holds ``export const T = () => <p>{`a``, a blank line, `` b`}</p> `` and a paragraph
- **THEN** the export covers both lines and the paragraph stays prose

### Requirement: Invalid export code is an error

A syntax error in an import or export block SHALL fail the file with an error that names its line.  
A block incomplete at the end of the file SHALL fail the file too.

#### Scenario: A syntax error in an export

- **WHEN** an MDX file holds `export const = 1` on line 3
- **THEN** parsing fails and the error names line 3

#### Scenario: An unclosed export at the end of the file

- **WHEN** an MDX file holds `export const a = {`, a blank line and a paragraph
- **THEN** parsing fails

### Requirement: Deep nesting does not crash hakkiri

hakkiri SHALL build the tree of a document with 1,000 levels of block quotes without exhausting the stack.  
It SHALL do the same for 1,000 levels of lists.

#### Scenario: A thousand block quotes

- **WHEN** a Markdown file holds a line of 1,000 `>` markers and a word
- **THEN** the tree holds a `blockquote` node 1,000 levels deep, with the `paragraph` node at the bottom

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
Doc line comments of the same type SHALL join across blank lines, as rustdoc reads them, and a line of code or a comment of another type SHALL end the run.

#### Scenario: A doc comment over three lines

- **WHEN** a Rust file holds three `///` lines, `/// First line`, `///` and `/// Second paragraph.`
- **THEN** the tree holds one `text.comment.doc.line` node with two `paragraph` children

#### Scenario: A fence across a blank line

- **WHEN** a Rust file holds `/// ```sh`, an empty line, `/// curl it` and `/// ```` `
- **THEN** the tree holds one `text.comment.doc.line` node, and `curl it` is in its code block

#### Scenario: A line of code ends the run

- **WHEN** a Rust file holds `/// ```sh`, `/// curl a`, `fn f() {}`, an empty line and `/// curl b`
- **THEN** the second `///` line is in a node of its own, read as prose

#### Scenario: Plain comments across a blank line

- **WHEN** a Rust file holds `// ```sh`, an empty line and `// curl c`
- **THEN** the two lines are in two nodes, and `curl c` is prose

### Requirement: Spans inside comments point into the file

A node inside a comment SHALL span the bytes of its text in the file, not in the text with markers stripped.

#### Scenario: A link in a doc comment

- **WHEN** a Rust file holds the line `/// See [the guide](https://g.example).` indented by four spaces
- **THEN** the `link` node spans the bytes of `[the guide](https://g.example)` in that line of the file

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

### Requirement: A Markdown tree counts code blocks

The tree of a Markdown or MDX file SHALL hold the number of its fenced and indented code blocks.  
The tree of a source file SHALL hold 0.

#### Scenario: Two fences and an indented block

- **WHEN** a Markdown file holds two fenced code blocks and one block indented by four spaces
- **THEN** the tree holds a code block count of 3

### Requirement: A node records its first-child place

The tree of a Markdown or MDX file SHALL record on each node whether it is the first child of the list item, quote or table row that holds it.  
The parser decides, from every child of that container.

#### Scenario: A paragraph after a fenced block in an item

- **WHEN** a Markdown file holds a list item whose first child is a fenced block and whose second child is the paragraph `After the fence.`
- **THEN** the tree holds a `paragraph` node for `After the fence.` that is not first in its item

#### Scenario: A paragraph that opens an item

- **WHEN** a Markdown file holds a list item whose first child is the paragraph `Opens the item.`
- **THEN** the tree holds a `paragraph` node for `Opens the item.` that is first in its item

### Requirement: A block takes the first place as Vale reads it

A fenced block, a table, an empty item and an HTML block SHALL take the first place when the HTML has an end tag that is not inline, or text outside its tags and comments.  
A thematic break, a link definition and any other HTML block SHALL leave it free, as in Vale 66bbb85, which reads HTML with the tokenizer of golang.org/x/net/html.

#### Scenario: A paragraph after a thematic break in a quote

- **WHEN** a Markdown file holds a quote whose first child is a thematic break and whose second child is the paragraph `After the rule.`
- **THEN** the tree holds a `paragraph` node for `After the rule.` that is first in its quote

#### Scenario: A paragraph after a comment and a div in a quote

- **WHEN** a Markdown file holds a quote whose first child is the HTML block `<!-- c --> <div>x</div>` and whose second child is the paragraph `After the div.`
- **THEN** the tree holds a `paragraph` node for `After the div.` that is not first in its quote

#### Scenario: A paragraph after a bare start tag in a quote

- **WHEN** a Markdown file holds a quote whose first child is the HTML block `<div>` and whose second child is the paragraph `After the tag.`
- **THEN** the tree holds a `paragraph` node for `After the tag.` that is first in its quote

#### Scenario: A paragraph after a processing instruction with a tag in it

- **WHEN** a Markdown file holds a quote whose first child is the HTML block `<?php echo "<div>"; ?>` and whose second child is the paragraph `After the block.`
- **THEN** the tree holds a `paragraph` node for `After the block.` that is not first in its quote

[tree-4]: ../../changes/archive/2026-11-10-document-tree/design.md#4-scopes-from-mdast-nodes
