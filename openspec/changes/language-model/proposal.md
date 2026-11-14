# Proposal

Queue row: `HKR-066` in [tasks.yaml][ledger].

## Motivation

hakkiri chose the reader of a file by its extension, from tables of its own in three places: the readers, the scope parser and the extension a reader gives.
A scope such as `text.js` compared strings, so it named an extension and not a language.
A file with no reader took the extension `txt`, so a rule scoped `text.txt` reached `.rst` and `README` files (row [HKR-066][ledger], found by a review of pull request 131).
A fenced code block was a gap with no language, so the comments of a ```` ```rust ```` example in a Markdown file were never read.

The operator ruled on 2026-11-13 that a file or an embedded segment has one language, with a name, aliases and extensions, distinct from a file extension, and that [langdex][langdex] holds the table, generated from GitHub Linguist.
On 2026-11-14 the operator asked that each fenced block be parsed as its language.

## Changes

- Each file has the langdex language of its path, its file name, or the interpreter of its first line.
- A reader is chosen by `formats`, then by the language of the file, then by `on_unknown`, so `.mts`, `.cts`, `.markdown` and a script that starts `#!/usr/bin/env node` get their reader.
- The last piece of a scope part names a language by any of its langdex names: `text.js` reaches `.js`, `.jsx`, `.mjs` and `.cjs` files, and `text.txt` reaches only plain-text files, never a `.rst` file or a `README`.
- A fenced code block takes the language its info string names.
  A fence of a language whose comments hakkiri reads is parsed by that language's grammar, and its comments enter the tree at their place in the file, each node with that language.
  A fence of Markdown, MDX or plain text stays a gap, as in Vale, since its text is an example.
- `formats` takes any name of a language hakkiri reads, such as `markdown` beside `md`.
- `READERS`, `Language::from_path`, `Language::extension` and the scope extension list read from langdex and list no extensions of their own.

## Capabilities

### Modified Capabilities

- `scope-selection`: a scope part names a language.
- `check-command`: how a file gets its language and its reader.
- `project-config`: the reader names `formats` takes.
- `document-tree`: the comments of a fenced code block.
- `vale-differences`: the comments of a fence, and a scope that names a language.

## Impact

- New dependency: `langdex` 0.0.1, MIT OR Apache-2.0, with no dependencies of its own; it adds about 425 KB to a release binary.
- Changed: `src/language.rs`, `src/scope.rs`, `src/select.rs`, `src/engine.rs`, `src/files.rs`, `src/formats.rs`, `src/suite.rs`, `src/markdown.rs`, `src/tree.rs`, their tests, `src/instructions.md`, the skill and `AGENTS.md`.
- Public API: `READERS` is gone, `Language` gains the langdex mapping, and `Node` gains a `language`.

[ledger]: ../../../tasks.yaml
[langdex]: https://github.com/victor-software-house/langdex
