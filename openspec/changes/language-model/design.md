# Design

## Context

hakkiri has seven readers: Markdown, MDX, Rust, TypeScript, TSX, JavaScript and plain text.
It chose a reader from a match on the extension in `Language::from_path`, gave each reader an extension in `Language::extension`, listed the reader names in `READERS`, and listed the extensions a scope part may end in in `EXTENSIONS`.
[langdex][langdex] 0.0.1 holds the GitHub Linguist table at a pinned commit: each language with its name, aliases, extensions, file names and interpreters, and the lookups `by_path`, `by_name`, `by_first_line` and `by_fence`.

## Decisions

### 1. A language and a reader are two things

A file's language is the langdex entry for it.
A reader is one of the seven parsers hakkiri has, and each reader reads one langdex language: Markdown, MDX, Rust, TypeScript, TSX, JavaScript and Text.
`Language` keeps naming the readers, and maps to and from its langdex entry by name.

Alternative: one enum for every language. It would copy the Linguist table that langdex already holds and would give most entries no reader.

### 2. The language of a file

A file has the language that `langdex::by_path` gives for its path, which reads the file name, then the extension.
A file that gives none has the language of the interpreter on its first line, through `langdex::by_first_line`, read from at most the first 4096 bytes.
Otherwise it has no language.
So `LICENSE` and `notes.txt` are Text, `Cargo.toml` is TOML, `README` has none, and `bin/tool` that starts `#!/usr/bin/env bash` is Shell.

Alternative: Linguist's content heuristics. They need a classifier and samples, and a project names a file the extension does not fit through `formats`.

### 3. The reader of a file

A file is read by the reader its `formats` entry gives, then by the reader of its language, then as `on_unknown` says.
A Text file therefore reads as plain text whatever `on_unknown` says, as Vale reads `.txt` as text.
A file whose language hakkiri has no reader for, or that has no language, falls to `on_unknown`, as before.

### 4. A scope part names a language

The last piece of a scope part is a language when the whole part is not a scope hakkiri defines, the rest is one, and `langdex::by_name` finds the piece among the names, aliases and extensions of its languages.
A part whose last piece names no language is an error, as an unknown extension was.
`text.comment.line` stays a scope, since the whole part is one, even if `line` were a language name.

A node is in a language when its own language is that language.
A node outside a fenced block has the languages of its file: the file's own language, and the language of its reader when `formats` chose that reader.
So `formats: {txt: md}` keeps `text.md` and `text.txt` both reaching `a.txt`, as before.
A file that `on_unknown` reads as plain text has only its own language, so `text.txt` no longer reaches `Cargo.toml` or a `README`.

Alternative: count the reader's language for every file. It is the defect of row `HKR-066`: every file read as plain text would be Text.

### 5. A fenced code block is parsed as its language

The info string of a fenced code block names its language through `langdex::by_fence`, so ```` ```rs ```` and ```` ```rust ```` are both Rust.
When hakkiri reads the comments of that language, it parses the code of the fence with that grammar, and the comments become nodes inside the node that holds the fence, as the comments of a source file become roots.
Each such node, and every node inside it, has that language, and a comment reads as Markdown, as decision 3 of `AGENTS.md` sets for every comment.
The fence stays a gap in the prose around it.

The code of a fence is the text the Markdown parser gives without its indentation and container markers.
Each line of it maps to the line of the file it came from, where the file line ends with it.
A fence with a line that does not map gives no comments, so a span never points at the wrong bytes.

A fence of Markdown, MDX or plain text gives no nodes, as in Vale, since its text is an example and not the prose of the document.
A fence with no info string, a language hakkiri reads no comments of, and an indented code block stay gaps.

Alternative: read the prose of a Markdown fence too. It would lint every example of the documentation, and Vale reads none of it.

### 6. The tables come from langdex

`Language::from_path` is `langdex::by_path` mapped to a reader.
`Language::extension` is the first extension of the reader's langdex entry, without its dot, which is the same `md`, `mdx`, `rs`, `ts`, `tsx`, `js` and `txt` as before.
`READERS` and `EXTENSIONS` are gone: `formats` takes any langdex name of a language hakkiri reads, and its error names the seven languages.

## Risks

- A Linguist update can move an extension to another language.
  langdex pins its Linguist commit and keeps the order it ranks an extension's languages in, so a change shows in its own pull request.
- Comments of a fence now draw alerts that Vale does not give.
  The `vale-differences` spec records it.

[langdex]: https://github.com/victor-software-house/langdex
