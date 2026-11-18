# Design

## Context

[`check-command`][check] reads a file only when its extension matches a language.  
Vale at [66bbb85][v-format] picks a reader from the `[formats]` section first and from a table of extensions second.  
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
Each run of lines that have text, between blank lines, is a `paragraph` child that spans from its first character to its last.  
Nothing is decoded: a fence, an emphasis mark or an entity is text.

So `text`, `paragraph`, `sentence` and `raw` scopes select what they select in Markdown, and a rule with another scope finds nothing.

Alternative: read plain text as Markdown.  
It lost because a `.txt` or `.toml` file contains underscores, asterisks and backticks that mean nothing, and Vale does not parse them either.

### 2. formats keeps Vale's lookup order

An entry for the file's extension comes first ([`formatFor`][v-for]).  
Then comes an entry whose key is the file's name, or a glob that matches its name or its path.  
Those come in the order of the keys as text.

A key that contains `*`, `?`, `[`, `{` or `/` is a glob.  
The entry beats the extension table, so `md: text` reads Markdown files as text.  
Sections match the path as it is.

At 66bbb85 Vale keeps the path with the reader's extension and matches nothing with it.  
The value is one of the seven readers, and any other word is an error that lists them.

Alternative: also accept Vale's extension aliases as values.  
It lost because the readers are the languages hakkiri has, and `txt` or `markdown` as a value would point to Vale's table.

### 3. A package may contain formats and the project wins

A package's settings file contains `formats`, because a package of rules for `.toml` files needs the mapping to run.  
The entries of every config add up, a later one replacing an earlier one with the same key, so the project overrides a package.

Vale takes the first value for `[formats]` and lets a package win ([package settings 5][settings-5]).  
That departure follows the rule the package-settings change set for every key, and the `vale-differences` spec contains its scenario.

### 4. on_unknown is a run option

`on_unknown` is `text`, `warn`, `error` or `ignore`, resolved as the flag, the variable, the config, then the default.  
A package's file may not set it, for the reason `gitignore` may not be set there.

It applies only to a file that a glob selects.  
A walk meets many files that no glob selects, and none of them draws a message.  
`warn` and `error` give the file and show how to map it.

Alternative: a failure by default.  
It lost because the point of the row is that a `*` glob lints the text files too.  
A failure would keep the refusal it replaces.

### 5. A binary file is skipped

A file read as plain text that contains a NUL (zero) byte in its first 8192 bytes is binary.  
It is skipped with no message, as ripgrep and typos skip binaries.

A file with bytes that are not valid UTF-8 is not binary: each such byte reads as one `?`.  
So every other byte keeps its offset, and a line and a column give the place Vale gives, which counts bytes.

Only a file read as text is tested: a `.md` or `.rs` file is read as it is.  
Vale has no such test in `NewFile` or `lintFile`, so it reads a binary file as lines.

Alternative: test every file.  
It lost because a file with a reader is the project's own statement that it is text.

### 6. A mapped file has two extensions

A scope may use either extension of a file.  
One is the file's own, and one is the reader's, which the tree records.  
So `text.md` and `~heading.md` select in a `.txt` file mapped to `md`, and `text.txt` still selects in it.

Vale compares a scope's extension with the file's own, `RealExt`, so there `text.md` selects nothing in that file.  
A file with no entry has its own extension and its reader's too, which is the same one for a Markdown or source file.

Alternative: only the reader's extension.  
It lost because a rule written for `.txt` files would stop applying to a `.txt` file that `formats` maps, where Vale applies it.

Alternative: only the file's own extension, as Vale does.  
It lost because the entry marks the file as Markdown, and a rule for Markdown should apply to it.

[check]: ../../../specs/check-command/spec.md
[v-format]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/format.go#L169-L196
[v-for]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/format.go#L203-L229
[v-lines]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/lint.go#L560-L563
[v-txt]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/lint.go#L508-L547
[settings-5]: ../2026-11-11-package-settings/design.md#5-scalars-the-project-wins
