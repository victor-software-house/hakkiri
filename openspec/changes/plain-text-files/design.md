# Design

## Context

[`check-command`][check] reads a file only when its extension names a language, and Vale at [66bbb85][v-format] picks a reader from the `[formats]` section first and from a table of extensions second.
Vale reads a file with no reader as lines of text in one block ([`lintLines`][v-lines]), and a `.txt` file through [`lintTxt`][v-txt].

## Goals / Non-Goals

Goals:

- A selected file with no reader is linted as prose.
- `formats` picks a reader the way Vale's does, and sets it from a config or a package.
- A project can turn the plain-text default off.

Non-goals:

- The manuscript reading of a `.txt` file with chapter lines, and Vale's other formats (reStructuredText, AsciiDoc and the rest, row `HKR-017` and later).
- A `Language` for each Vale extension alias such as `.markdown`: a `formats` entry maps one.

## Decisions

### 1. Plain text is one raw root of paragraphs

A plain-text file gives one `raw` root over the whole file.
Each run of lines that hold something, between blank lines, is a `paragraph` child that spans from its first character to its last.
Nothing is decoded: a fence, an emphasis mark or an entity is text.
So `text`, `paragraph`, `sentence` and `raw` scopes select what they select in Markdown, and a rule with another scope finds nothing.
Alternative: read plain text as Markdown.
It lost because a `.txt` or `.toml` file holds underscores, asterisks and backticks that mean nothing, and Vale does not parse them either.

### 2. formats keeps Vale's lookup order

An entry named by the file's extension comes first, then, in the order of the keys as text, an entry whose key is the file's name, or a glob that matches its name or its path ([`formatFor`][v-for]).
A key that holds `*`, `?`, `[`, `{` or `/` is a glob.
The entry beats the extension table, so `md: text` reads Markdown files as text.
Sections match the path as it is.
At 66bbb85 Vale keeps the path with the reader's extension and matches nothing with it.
The value is one of the seven readers, and any other word is an error that lists them.
Alternative: also accept Vale's extension aliases as values.
It lost because the readers are the languages hakkiri has, and `txt` or `markdown` as a value would name Vale's table.

### 3. A package may hold formats and the project wins

A package's settings file holds `formats`, because a package of rules for `.toml` files needs the mapping to run.
The entries of every config add up, a later one replacing an earlier one with the same key, so the project overrides a package.
Vale takes the first value for `[formats]` and lets a package win ([package settings 5][settings-5]).
That departure follows the rule the package-settings change set for every key, and the `vale-differences` spec holds its scenario.

### 4. on_unknown is a run option

`on_unknown` is `text`, `warn`, `error` or `ignore`, resolved as the flag, the variable, the config, then the default, and a package's file may not set it, for the reason `gitignore` may not be set there.
It applies only to a file that a glob selects: a walk meets many files that no glob selects, and none of them draws a message.
`warn` and `error` name the file and say how to map it.
Alternative: a failure by default.
It lost because the point of the row is that a `*` glob lints text files, and a failure would stay the refusal it replaces.

### 5. A binary file is skipped

A file read as plain text is binary and skipped with no message when its first 8192 bytes hold a NUL byte or are not valid UTF-8, since the run could not read it as text.
A character that the 8192-byte cut splits is not invalid.
ripgrep and typos skip a file with a NUL byte as binary.
Only a file read as text is tested: a `.md` or `.rs` file is read as it is.
Vale has no such test in `NewFile` or `lintFile`, so it reads a binary file as lines.
Alternative: test every file.
It lost because a file with a reader is the project's own statement that it is text.

### 6. A mapped file takes the extension of the reader

A `formats` entry that gives the reader also gives the extension the scopes see: `text.md` and `~heading.md` select in a `.txt` file mapped to `md`.
Vale compares a scope's extension with the file's own, `RealExt`, so there `text.md` selects nothing in that file.
A file with no entry keeps its own extension, as in Vale.
Alternative: keep the file's own extension, as Vale does.
It lost because the entry marks the file as Markdown, and a rule for Markdown should apply to it.

[check]: ../../specs/check-command/spec.md
[v-format]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/format.go#L169-L196
[v-for]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/format.go#L203-L229
[v-lines]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/lint.go#L560-L563
[v-txt]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/lint.go#L508-L547
[settings-5]: ../archive/2026-11-11-package-settings/design.md#5-scalars-the-project-wins
