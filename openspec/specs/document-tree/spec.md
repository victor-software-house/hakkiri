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

For Rust, TypeScript, JavaScript, Go, Swift, YAML, TOML and Shell files, hakkiri SHALL put only comment text in the tree, scoped `text.comment.line`, `text.comment.block`, `text.comment.doc.line` or `text.comment.doc.block` as the rule-files spec states.  
The `#!` line that opens a script SHALL be no comment.

#### Scenario: A Rust block comment

- **WHEN** a Rust file holds `/* A note. */` on its first line
- **THEN** the tree holds a `text.comment.block` node whose child `paragraph` spans bytes 3 to 10, the text `A note.`

### Requirement: A grammar finds the comments of a source file

Each language SHALL find its comments with its tree-sitter grammar, so a comment marker inside a string, a raw string, a multi-line string, a heredoc or a block scalar opens no comment.  
A Swift block comment SHALL nest.

#### Scenario: A YAML comment beside strings

- **WHEN** a YAML file holds `a: "x # y" # note` and a block scalar `b: |` whose next line starts with `#`
- **THEN** the tree holds one comment node for `note`, and no node holds `x # y` or the line of the block scalar

#### Scenario: A shell comment beside a heredoc

- **WHEN** a Shell file holds `#!/usr/bin/env bash`, `echo ${#a} # count`, and a heredoc whose body is `# text`
- **THEN** the tree holds one comment node for `count`

#### Scenario: A nested Swift comment

- **WHEN** a Swift file holds `/* outer /* inner */ still outer */`
- **THEN** the tree holds one `text.comment.block` node whose text holds `still outer`

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

### Requirement: A fenced code block takes a language

A fenced code block SHALL take the langdex language that the first word of its info string names, up to a comma, so `rs` and `rust`, or `md` and `markdown`, name one language, and `rust,no_run` names Rust.  
The fence SHALL stay a gap in the prose around it.

#### Scenario: A rustdoc fence

- **WHEN** a Markdown file holds a fence opened with ```` ```rust,no_run ```` around `// A very good start.`, and a rule flags `very`
- **THEN** hakkiri reports `very` in the comment

#### Scenario: An alias of the language

- **WHEN** a Markdown file holds one fence opened with ```` ```rs ```` and one opened with ```` ```rust ````, each around `// A very good start.`, and a rule flags `very`
- **THEN** hakkiri reports `very` once in each fence

### Requirement: A fence gives the comments of its language

When hakkiri reads the comments of a fence's language, it SHALL parse the code of the fence with that language's grammar.  
Each comment SHALL be a node inside the node that holds the fence, read as the comments of a source file are.  
Each span of those nodes SHALL be the bytes of the Markdown or MDX file, and each such node, and every node inside it, SHALL have that language.  
A fence whose code does not map line for line onto the file SHALL give no nodes.

#### Scenario: A Rust fence

- **WHEN** a Markdown file holds `Intro.`, then a fence opened with ```` ```rust ```` around `// A very good start.` and `let x = 1;`, and a rule flags `very`
- **THEN** hakkiri reports `very` on the line of the comment, at its column in the file
- **AND** the comment node has the language Rust

#### Scenario: A fence in a list item

- **WHEN** a list item holds a fence indented by three spaces around `// A very good start.`
- **THEN** hakkiri reports `very` at its column in the file, past the indentation

#### Scenario: A scope for the fence's language

- **WHEN** a rule has `scope: text.comment.rs` and `tokens: [marker]`, and a Markdown file holds a ```` ```rust ```` fence around `// A marker.` and a ```` ```js ```` fence around `// A marker.`
- **THEN** hakkiri reports one alert, in the Rust fence

### Requirement: Other fences give no nodes

A fence of Markdown, MDX or Text, a fence with no info string, a fence of a language whose comments hakkiri does not read, and an indented code block SHALL give no nodes.

#### Scenario: A Markdown fence

- **WHEN** a fence opened with ```` ```md ```` holds `A very good start.`, and a rule flags `very`
- **THEN** hakkiri reports nothing for the fence

#### Scenario: A fence of a language with no comment reader

- **WHEN** a fence opened with ```` ```python ```` holds `# A very good start.`, and a rule flags `very`
- **THEN** hakkiri reports nothing for the fence

### Requirement: A comment addressed to a tool is not prose

hakkiri SHALL leave out of the tree a comment that a tool reads and a person does not, by the patterns of its language, which the scenarios list.  
A directive SHALL end a run of line comments, so the line after it starts a new comment node.

#### Scenario: A Go directive

- **WHEN** a Go file holds `//go:build linux` and, on the next line, `// A note.`
- **THEN** the tree holds one comment node, for `A note.`
- **AND** a comment that opens with `go:`, `line`, `export`, `extern`, `nolint`, `lint:ignore`, or a lowercase name, a colon and a letter or digit, is left out the same way

#### Scenario: A TypeScript switch

- **WHEN** a TypeScript file holds `// eslint-disable-next-line no-console` and, on the next line, `// A note.`
- **THEN** the tree holds one comment node, for `A note.`
- **AND** a line comment that opens with `eslint`, `@ts-`, `prettier-ignore`, `biome-ignore`, `istanbul`, `c8`, `v8`, `@flow`, `@jsx`, `jshint` or `jslint` is left out, in TypeScript and JavaScript

#### Scenario: A bundler hint

- **WHEN** a JavaScript file holds `/* webpackChunkName: "a" */`
- **THEN** the tree holds no node for it
- **AND** a block comment that opens with `eslint`, `globals`, `jshint`, `jslint`, `istanbul`, `c8`, `@flow`, `@jsx`, `@__PURE__` or `#__PURE__` is left out the same way

#### Scenario: A Swift tool

- **WHEN** a Swift file holds `// swiftlint:disable all`, `// swiftformat:disable indent` or `// sourcery: AutoEquatable`
- **THEN** the tree holds no node for any of them

#### Scenario: A YAML schema line

- **WHEN** a YAML file holds `# yaml-language-server: $schema=./a.json` on its first line, or `# yamllint disable`
- **THEN** the tree holds no node for either

#### Scenario: A TOML schema line

- **WHEN** a TOML file holds `#:schema ./a.json`
- **THEN** the tree holds no node for it

#### Scenario: A shell lint switch

- **WHEN** a Shell file holds `# shellcheck disable=SC2086`
- **THEN** the tree holds no node for it

### Requirement: A doc link and a doc name draw no alert

hakkiri SHALL drop an alert whose match lies wholly in a doc name or a doc link.  
A doc name is the name of the declaration that a Go doc comment opens with.  
A doc link is a link in brackets of a Go or Rust comment, or an inline link tag of a TypeScript or JavaScript comment.  
It SHALL leave the text of the node as written, so counts and sentences do not change.  
A bracket SHALL be a link when no `(`, `[` or `:` follows it.

#### Scenario: A Go link and name

- **WHEN** a rule has `scope: text`, `nonword: true` and `tokens: ['[A-Za-z]+']`, and a Go file holds `// Run starts it, see [Stop].` above `func Run() {}`
- **THEN** hakkiri reports `starts`, `it` and `see`, and none for `Run` or `Stop`

#### Scenario: A link that has a target

- **WHEN** a Go file holds `// See [Stop](https://example.com) now.`
- **THEN** `Stop` draws an alert

#### Scenario: A Rust path

- **WHEN** a Rust file holds `/// See [crate::Name] now.`
- **THEN** no alert lies in `crate::Name`

#### Scenario: A JSDoc link

- **WHEN** a TypeScript file holds `/** See {@link Other|label} now. */`
- **THEN** no alert lies in `{@link Other|label}`
- **AND** the tags `{@linkcode}`, `{@linkplain}` and `{@tutorial}` are links the same way

#### Scenario: What a link names

- **WHEN** a Go link holds an identifier with dots and an optional `*`, such as `[*pkg.Type]`, or a Rust link holds a path with `::` or a code span, such as ``[`Type`]``
- **THEN** no alert lies in the brackets

### Requirement: More JSDoc tags name something

In a `/** */` comment, the name after a tag SHALL carry no prose scope for `@see`, `@event`, `@member`, `@var`, `@name`, `@alias`, `@memberof`, `@augments`, `@extends`, `@mixes`, `@borrows`, `@function`, `@func`, `@method`, `@class`, `@constructor`, `@namespace` and `@module`, as it does after `@param`.  
The same SHALL hold for `@enum`, `@external`, `@host`, `@kind`, `@listens`, `@fires`, `@emits`, `@this`, `@requires`, `@implements` and `@satisfies`.

#### Scenario: A see tag

- **WHEN** a TypeScript file holds `/** Text.` on one line and `* @see Hotel The place.` on the next
- **THEN** the tree holds prose for `Text.` and `The place.`, and none for `@see` or `Hotel`

#### Scenario: A see tag with a link

- **WHEN** a JavaScript file holds `* @see {@link Other} for more.` in a `/** */` comment
- **THEN** the tree holds prose for `{@link Other} for more.`, and no name is taken from the link

### Requirement: A line comment keeps the indentation of its text

For Go, YAML, TOML and Shell files, hakkiri SHALL remove from each line of a line comment only the first space after its marker.  
A line indented further SHALL stay indented, so Markdown reads four spaces or a tab as a code block.  
For Rust, TypeScript, JavaScript and Swift it SHALL keep removing the indentation that every line shares.

#### Scenario: A Go code block

- **WHEN** a Go file holds `// Run it:`, `//`, and `//	go run main.go`
- **THEN** the tree holds a code block for `go run main.go`, and no prose for it

#### Scenario: A YAML example

- **WHEN** a YAML file holds `# Use:`, `#`, and `#     "git::https://a.example"`
- **THEN** the tree holds a code block for the quoted address
