# Proposal

Queue row: `HKR-017` in [tasks.yaml][ledger].

## Motivation

[HKR-017][ledger] promises the comments of Vale's languages and of Bash, HCL, Kotlin, Nix, Solidity, YAML and TOML, with the cost in binary size measured before it ships (operator, 2026-11-11).
This change is its first tranche, and builds what the rest reuse.
The first tranche is Go, Swift, Bash, YAML and TOML, because the repositories of the ctl group use them.
A count on 2026-11-12 found 140 TOML, 118 YAML and 69 shell files in the eleven repositories, with test fixtures and snapshots left out.
The project types of the shared template add Go modules and Swift packages.

hakkiri reads the comments of Rust, TypeScript and JavaScript files only, and of fenced blocks of those languages.
It reads every other file the config selects as plain text, so a `Cargo.toml`, a workflow or a script is linted as prose from its first line to its last.
Vale reads the comments of Go files, and of YAML and TOML files when a view names them.
It reads a Swift file with its C grammar and reads no shell.
Its comment lexer also drops the comments addressed to tools, and masks the links and names of doc comments, which hakkiri did for no language.
A probe of Vale at `66bbb85` against hakkiri, in a box, found each of these differences, and the design holds their causes.

## Changes

- Five readers join Rust, TypeScript, TSX and JavaScript: Go, Swift, YAML, TOML and Shell.
  A file reaches one through its langdex language, as the `language-model` change sets, so `.yml`, `.bash`, `.zsh`, a script that starts `#!/bin/sh` and a `bashrc` file reach them, and `formats` can name them.
  A fenced code block of one of these languages gives its comments too.
- Each reads only comments, through its tree-sitter grammar, so a string, a raw string, a heredoc and a block scalar are never prose.
- A comment addressed to a tool is not prose: Vale's patterns for Go, TypeScript and JavaScript, and the directives of the tools of the other languages.
- A link in brackets in a Go or Rust doc comment, the name a Go doc comment opens with, and an inline `{@link}` of JSDoc draw no alert, as in Vale.
  More JSDoc tags take a name, as in Vale.
- A line comment of a Go, YAML, TOML or Shell file keeps the indentation of its text after the first space, so an indented example stays code.
- Swift is read by the Swift grammar, which Vale does not use, and the cases where the two differ are recorded in `vale-differences`.
- The row stays open for the other languages of its list, each one a row of the dialect table with its fixture.

## Capabilities

### Modified Capabilities

- `document-tree`: the languages whose comments enter the tree, directives, masks and indentation.
- `rule-files`: the scopes of the comments of the new languages.
- `check-command`: the readers a file's language reaches.
- `project-config`: the readers `formats` takes.
- `vale-differences`: the differences of this change.

## Impact

- New dependencies: `tree-sitter-bash`, `tree-sitter-go`, `tree-sitter-swift`, `tree-sitter-toml-ng` and `tree-sitter-yaml`, all under the MIT license.
- `Tree` gains a public field `masks`.
- A file that a glob selects and whose language is one of the five is read for its comments only, where it was read as plain text.
- Changed: `src/comments.rs`, `src/jsdoc.rs`, `src/language.rs`, `src/lib.rs`, `src/tree.rs`, `src/engine.rs`, the operator instructions, the skill, the README and the tests.

[ledger]: ../../../../tasks.yaml
