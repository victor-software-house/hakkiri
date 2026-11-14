## MODIFIED Requirements

### Requirement: Source files give the tree their comments only

For Rust, TypeScript, JavaScript, Go, Swift, YAML, TOML and Shell files, hakkiri SHALL put only comment text in the tree, scoped `text.comment.line`, `text.comment.block`, `text.comment.doc.line` or `text.comment.doc.block` as the rule-files spec states.  
Each language SHALL find its comments with its tree-sitter grammar, so a comment marker inside a string, a raw string, a multi-line string, a heredoc or a block scalar opens no comment.  
A Swift block comment SHALL nest.  
The `#!` line that opens a script SHALL be no comment.

#### Scenario: A Rust block comment

- **WHEN** a Rust file holds `/* A note. */` on its first line
- **THEN** the tree holds a `text.comment.block` node whose child `paragraph` spans bytes 3 to 10, the text `A note.`

#### Scenario: A YAML comment beside strings

- **WHEN** a YAML file holds `a: "x # y" # note` and a block scalar `b: |` whose next line starts with `#`
- **THEN** the tree holds one comment node for `note`, and no node holds `x # y` or the line of the block scalar

#### Scenario: A shell comment beside a heredoc

- **WHEN** a Shell file holds `#!/usr/bin/env bash`, `echo ${#a} # count`, and a heredoc whose body is `# text`
- **THEN** the tree holds one comment node for `count`

#### Scenario: A nested Swift comment

- **WHEN** a Swift file holds `/* outer /* inner */ still outer */`
- **THEN** the tree holds one `text.comment.block` node whose text holds `still outer`

## ADDED Requirements

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

hakkiri SHALL drop an alert whose match lies wholly in the name of a declaration that a Go doc comment opens with, in a link in brackets of a Go or Rust comment, or in an inline link tag of a TypeScript or JavaScript comment.  
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

In a `/** */` comment, the name after a tag SHALL carry no prose scope for `@see`, `@event`, `@member`, `@var`, `@name`, `@alias`, `@memberof`, `@augments`, `@extends`, `@mixes`, `@borrows`, `@function`, `@func`, `@method`, `@class`, `@constructor`, `@namespace`, `@module`, `@enum`, `@external`, `@host`, `@kind`, `@listens`, `@fires`, `@emits`, `@this`, `@requires`, `@implements` and `@satisfies`, as it does after `@param`.

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
