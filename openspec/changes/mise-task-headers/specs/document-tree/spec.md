# Spec Delta

## MODIFIED Requirements

### Requirement: A comment addressed to a tool is not prose

hakkiri SHALL leave out of the tree a comment that a tool reads and a person does not.  
Each language has its own patterns for such comments, which the scenarios list.  
A directive SHALL end a run of line comments, so the line after it starts a new comment node.

#### Scenario: A Go directive

- **WHEN** a Go file contains `//go:build linux` and, on the next line, `// A note.`
- **THEN** the tree contains one comment node, for `A note.`
- **AND** a comment that opens with `go:`, `line`, `export`, `extern`, `nolint`, `lint:ignore`, or a lowercase name, a colon and a letter or digit, is left out the same way

#### Scenario: A TypeScript switch

- **WHEN** a TypeScript file contains `// eslint-disable-next-line no-console` and, on the next line, `// A note.`
- **THEN** the tree contains one comment node, for `A note.`
- **AND** a line comment that opens with `eslint`, `@ts-`, `prettier-ignore`, `biome-ignore`, `istanbul`, `c8`, `v8`, `@flow`, `@jsx`, `jshint` or `jslint` is left out, in TypeScript and JavaScript

#### Scenario: A bundler hint

- **WHEN** a JavaScript file contains `/* webpackChunkName: "a" */`
- **THEN** the tree contains no node for it
- **AND** a block comment that opens with `eslint`, `globals`, `jshint`, `jslint`, `istanbul`, `c8`, `@flow`, `@jsx`, `@__PURE__` or `#__PURE__` is left out the same way

#### Scenario: A Swift tool

- **WHEN** a Swift file contains `// swiftlint:disable all`, `// swiftformat:disable indent` or `// sourcery: AutoEquatable`
- **THEN** the tree contains no node for any of them

#### Scenario: A YAML schema line

- **WHEN** a YAML file contains `# yaml-language-server: $schema=./a.json` on its first line, or `# yamllint disable`
- **THEN** the tree contains no node for either

#### Scenario: A TOML schema line

- **WHEN** a TOML file contains `#:schema ./a.json`
- **THEN** the tree contains no node for it

#### Scenario: A shell lint switch

- **WHEN** a Shell file contains `# shellcheck disable=SC2086`
- **THEN** the tree contains no node for it

#### Scenario: A mise task header

- **WHEN** a Shell file contains `#MISE description="kilo task"`, `# [MISE] alias="lima"` and `#USAGE arg "<mike>"`, and on the next line `# november after`
- **THEN** the tree contains one comment node, for `november after`
