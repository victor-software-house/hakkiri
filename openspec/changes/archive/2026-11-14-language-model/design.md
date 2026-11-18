# Design

## Context

hakkiri has seven readers: Markdown, MDX, Rust, TypeScript, TSX (TypeScript with JavaScript XML), JavaScript and plain text.
It used to keep its own tables for them:

- `Language::from_path` chose a reader by matching the extension.
- `Language::extension` gave each reader an extension.
- `READERS` listed the reader names.
- `EXTENSIONS` listed the extensions a scope part may end in.

[langdex][langdex] 0.0.1 contains the GitHub Linguist table at a pinned commit.
It has each language with its name, aliases, extensions, file names and interpreters.
It also has the lookups `by_path`, `by_name`, `by_first_line` and `by_fence`.

## Decisions

### 1. A language and a reader are two things

A file's language is the langdex entry for it.
A reader is one of the seven parsers hakkiri has.
Each reader reads one langdex language: Markdown, MDX, Rust, TypeScript, TSX, JavaScript and Text.
`Language` still lists the readers, and maps to and from its langdex entry by name.

Alternative: one enum for every language.
It would copy the Linguist table that langdex already contains, and most entries would have no reader.

### 2. The language of a file

A file has the language that `langdex::by_path` gives for its path, which reads the file name, then the extension.
If that gives none, the file has the language of the interpreter on its first line.
`langdex::by_first_line` finds it, from at most the first 4096 bytes.
Otherwise the file has no language.

So `LICENSE` and `notes.txt` are Text, `Cargo.toml` is TOML, and `README` has none.
A `bin/tool` that starts `#!/usr/bin/env bash` is Shell.

Alternative: Linguist's content heuristics.
They need a classifier and samples.
A project can already map a file whose extension does not fit through `formats`.

### 3. The reader of a file

A file is read by the reader its `formats` entry gives.
If there is none, the reader of its language reads it, and after that `on_unknown` applies.

So a Text file reads as plain text whatever `on_unknown` is set to, as Vale reads `.txt` as text.
A file whose language hakkiri has no reader for, or that has no language, falls to `on_unknown`, as before.

### 4. A scope part names a language

The last piece of a scope part is a language when three things are true:

- the whole part is not a scope hakkiri defines;
- the rest of the part is one;
- `langdex::by_name` finds the piece among the names, aliases and extensions of its languages.

A part whose last piece matches no language is an error, as an unknown extension was.
`text.comment.line` stays a scope, since the whole part is one, even if `line` were a language name.

A node is in a language when its own language is that language.
A node outside a fenced block has the languages of its file.
These are the file's own language, and the language of its reader when `formats` chose that reader.

So `formats: {txt: md}` keeps `text.md` and `text.txt` both reaching `a.txt`, as before.
A file that `on_unknown` reads as plain text has only its own language.
So `text.txt` no longer reaches `Cargo.toml` or a `README`.

Alternative: count the reader's language for every file.
That is the defect of row `HKR-066`: every file read as plain text would be Text.

### 5. Fenced code is parsed with the grammar of the fence language

The info string of a fenced code block gives its language through `langdex::by_fence`.
So ```` ```rs ```` and ```` ```rust ```` are both Rust.
The first word counts up to a comma, as rustdoc writes ```` ```rust,no_run ````.

hakkiri reads the comments of some languages.
When a fence uses one of them, hakkiri parses its code with that language's grammar.
The comments become nodes inside the node that contains the fence, as the comments of a source file become roots.

Each such node, and every node inside it, has that language.
A comment reads as Markdown, as decision 3 of `AGENTS.md` sets for every comment.
The fence stays a gap in the prose around it.

The code of a fence is the text the Markdown parser gives without its indentation and container markers.
Each line of it maps to the line of the file it came from, where the file line ends with it.
A fence with a line that does not map gives no comments, so a span never points at the wrong bytes.

A fence of Markdown, MDX or plain text gives no nodes, as in Vale.
Its text is an example and not the prose of the document.
These code blocks stay gaps:

- a fence with no info string;
- a fence in a language whose comments hakkiri does not read;
- an indented code block.

Alternative: read the prose of a Markdown fence too.
It would lint every example of the documentation, and Vale reads none of it.

### 6. The tables come from langdex

`Language::from_path` is `langdex::by_path` mapped to a reader.
`Language::extension` is the first extension of the reader's langdex entry, without its dot.
These are the same `md`, `mdx`, `rs`, `ts`, `tsx`, `js` and `txt` as before.
`READERS` and `EXTENSIONS` are gone.
`formats` takes any langdex name of a language hakkiri reads, and its error lists the seven languages.

## Risks

- A Linguist update can move an extension to another language.
  langdex pins its Linguist commit and keeps the order it ranks an extension's languages in, so a change shows in its own pull request.
- Comments of a fence now draw alerts that Vale does not give.
  The `vale-differences` spec records it.

[langdex]: https://github.com/victor-software-house/langdex
