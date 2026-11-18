# Design

## Context

Vale at `66bbb85` reads code through tree-sitter grammars that it ships as WebAssembly modules, 22 of them ([names][v-names]).  
A table maps an extension to a language ([languageFor][v-lang]), and an extension with no grammar falls back to the comment delimiters of a regular expression.

hakkiri reads the grammars of Rust, TypeScript and JavaScript through the Rust crates of the same parsers, in [`comments.rs`][comments].

The probe of 2026-11-12 ran Vale built from `66bbb85`, with the grammars of its `grammars/build.sh`, and hakkiri over the same files in a box.  
A rule of `existence` type that matches every word made each alert one word of comment text.  
So the two tools could be compared word by word.

The inputs were 23 hand-written files, each with one kind of comment, and 640 files of the ctl repositories.  
They also took 278 Go files of the Vale tree and 122 Swift files of a public package.

## Decisions

### 1. The first tranche is the languages the repositories use

A count on 2026-11-12 covered the eleven repositories of the group, with fixtures and snapshots left out.  
It found 252 Rust, 61 TypeScript, 140 TOML, 118 YAML and 69 shell files.  
The project types of the shared template add Go modules and Swift packages.

All five new languages are on the list of the row.  
Go and Swift are among Vale's, Bash and YAML are among the six it adds, and TOML is the operator's later request.

Rego files and Nushell scripts also appear in the repositories, and no crate on crates.io contains a grammar for either.  
So those files stay plain text until one exists.

The five grammars add 5.9 MB to the release binary, 15,264,456 to 21,171,528 bytes.  
They add 6 seconds to a 76 second release build in a box with 4 vCPUs.

The size of each crate's archive gives this order:

| Grammar | Archive | In hakkiri |
|:--|:--|:--|
| Swift | 5.8 MB | new |
| TypeScript with TSX | 3.4 MB | linked already |
| Bash | 1.5 MB | new |
| Rust | 1.4 MB | linked already |
| JavaScript | 0.6 MB | linked already |
| Go | 0.36 MB | new |
| YAML | 0.24 MB | new |
| TOML | 0.06 MB | new |

The archives overstate the binary, whose growth is 5.9 of their 8.0 MB, and the order is the same.

Alternative: every language of the row in one change.  
It lost because the size must be measured before the row ships, and one grammar, Swift, is 70 percent of this tranche.  
Each of the other languages is one row of the dialect table and one fixture, and the numbers above are the cost to expect.

### 2. A dialect is one table row

Only five things differ between languages: the grammar, the node kinds that contain a comment, and the marker (`//` or `#`).  
The other two are the directive pattern, and whether `///` is a doc comment.  
The grouping of adjacent lines, the stripping of markers and the Markdown reading stay as they were.

Alternative: a module for each language.  
It lost because the five modules would repeat the same grouping and stripping code.

### 3. The lexer is the grammar's, probed against Vale

The fixtures have line, block, doc and nested comments, strings that look like comments, raw strings and heredocs.  
Over them hakkiri and Vale gave the same alerts for Go, YAML, TOML, TypeScript and JavaScript.  
The exceptions are the drop of decision 9 and the cases of decisions 5, 7 and 8.

The strings that contain a comment marker were a Go string, raw string and rune.  
They were also a YAML quoted scalar, block scalar and plain scalar with `#`, and a TOML string, literal string and multi-line strings.

Swift differs.  
Vale maps `.swift` to its C grammar ([formats][v-formats]), which has no nested block comment, no `#"` raw string and no multi-line string.

A nested comment ends at its first `*/` in Vale, and a `//` inside `##"x "# // y"##` or inside `"""` is read as a comment.  
A comment inside an interpolation is missed.  
hakkiri reads each as the Swift grammar does.

Shell has no Vale reader.  
Its table has no `.sh`, and the grammar `bash` is built and never used, so a shell file reads as plain text, or as Markdown with `sh = md` in `[formats]`.  
hakkiri reads the `#` comments, and not the `#!` line, `$#`, `${#a}`, `${a#b}`, a `#` in a string, a heredoc or an escaped `\#`.

Alternative for Swift: the C grammar, as Vale does.  
It lost because it reads code inside strings as prose and loses text after a nested comment.

### 4. YAML and TOML comments need no view

Vale reads the comments of a data file only when a view lists the file ([lint dispatch][v-dispatch], [data comments][v-data]).  
Otherwise it reads the whole file as plain lines.

hakkiri reads the comments of every file a glob selects as a YAML or TOML file, and nothing else.  
A key or a value is no prose to a rule that reads comments.

Alternative: read the values of the file too, as a Vale view does.  
It lost because the request is comment prose, and a view that picks values is a separate feature with its own selector language.

### 5. A comment for a tool is not prose

Vale drops a comment that a tool reads from the comments of Go, TypeScript, JavaScript and four other languages ([directives][v-directive]).  
It also drops a `#!` line at the top of a file ([dropShebang][v-shebang]).  
hakkiri dropped none, so `// eslint-disable-next-line` read as a sentence.  
It now uses Vale's patterns for Go, TypeScript and JavaScript.

For Swift, YAML, TOML and shell it lists the directives of the tools these repositories run: `swiftlint:`, `swiftformat:`, `sourcery:`, `yaml-language-server:`, `yamllint`, `#:schema` and `shellcheck`.  
Vale reads them as sentences, such as `yaml language server schema https github com ...`, and that was 110 alerts in 7 files of the repositories.

Alternative: drop every comment of the shape `name: value`.  
It lost because `Note: this holds` is prose.

### 6. A doc link and a doc name are masked at the alert

Vale drops an alert whose match lies wholly in a mask ([Masked][v-masked]).  
One mask is the name a Go doc comment opens with, taken from the declaration the comment precedes ([goDocName][v-docname]).  
The others are a link in brackets ([GoDoc][v-godoc]), a rustdoc link, and the inline tags and names of JSDoc.

hakkiri did this for JSDoc tags only, by cutting them out of the text.  
It now keeps the masked file ranges on the tree, and the engine drops an alert whose match lies wholly in one.  
The text, the words and the sentences stay as written, so a count has the words of the link, as in Vale.

JSDoc's `@see` and the other tags that point to something take a name, as Vale's list does.  
A name does not start at an inline tag.  
Vale reads the `{@link Other}` of `@see {@link Other} for more.` as a type and `for` as the name, and hakkiri keeps `for` as prose.

Alternative: cut the link out of the text, as for a JSDoc tag.  
It lost because `See [Stop] for more.` would read as two sentences, one that ends before the link and one that starts after it, and the counts would move.

The grammar of the Go crate puts a block's first comment before the statement list.  
The grammar of Vale puts it before the first statement.  
A comment above `var hit string` inside a function is a doc comment of `hit` in Vale, so hakkiri reads through the list.

### 7. A line comment keeps the indentation of the text

Go removes the first space of a line comment and no more, and a line indented further is a code block of the comment.

Vale takes every leading space off each line of a run after the first ([appendLine][v-append]), so `///     cargo run` is a sentence.  
hakkiri removed the indentation that every line shares, as rustdoc does, which turned a comment that contains only an indented example into prose.

For Go, YAML, TOML and shell, hakkiri now removes the first space of each line and no more.  
For Rust, TypeScript, JavaScript and Swift it keeps the rule of rustdoc, which keeps an example code when a prose line sits beside it.  
Vale reads the indented line of all of them as prose, so the difference is recorded for all.

Alternative: Vale's trim.  
It lost because code reads as prose.  
100 lines of the 122 Swift files, and the examples of Go and TOML files, are indented code that Vale reads as sentences.  
The rule of rustdoc for Go lost too, since it made 25 lines of tab-indented Go examples prose.

### 8. A column counts characters

In Vale, a comment that trails code after a character of several bytes has columns in bytes.  
So the columns of the words after `"’"` are 2 too high.  
hakkiri gives columns in characters, as it does for every file.

### 9. What the files show

Vale drops an alert that has the place of another ([code-span-size design 6][code-span-size-6]).  
The probe shows the same drop in comments, which Markdown files already showed.

Over the real files the drop accounts for most of the difference.  
So the table below is against a Vale built with that check off, as the design of the word counts does.

Over the real files, the alerts that only one tool gave were, as Vale only and hakkiri only:

| Files | Count | Vale as built | Vale with the check off |
|:--|--:|:--|:--|
| Go | 278 | 13 and 324 | 13 and 92 |
| Rust | 252 | 3 and 2,433 | 3 and 18 |
| Swift | 122 | 315 and 539 | 321 and 6 |
| TOML | 140 | 39 and 381 | 39 and 4 |
| TypeScript | 61 | 0 and 81 | 0 and 0 |
| YAML, `.yaml` | 58 | 110 and 151 | 110 and 0 |
| YAML, `.yml` | 60 | 0 and 344 | 0 and 3 |
| Shell | 69 | no Vale reader | no Vale reader |

Each difference that is left has a cause, found on its own input:

- Indented examples: Vale reads them as sentences (decision 7).  
  They give 315 Vale-only rows of Swift in 100 lines, and a line of Go and 3 lines of TOML.
- Tool directives: the 110 Vale-only rows of YAML are the `yaml-language-server` line of 7 files (decision 5).
- Columns after a multi-byte character: 10 rows of Go and 12 of Swift, in comments that trail code (decision 8).
- Markdown reading, the same as in Markdown files.  
  The Go rows contain inline HTML such as `<script>`, a code span beside a word, and a triple backtick in a sentence.  
  The Rust rows contain a code span beside a word.  
  Vale's converter and hakkiri's parser read these otherwise, and `vale-differences` already records the Markdown cases.
- One line of a workflow contains a `${{ }}` expression and gives 3 rows, and 4 TOML rows contain a reference link and a code span.

A comment that opens a function body is the one place where the two grammars of Go differ, as decision 6 states.  
The probe found it as a `hit` that only hakkiri reported.

### 10. Readers follow the language

A reader reads one language of langdex, and a file reaches it by the language of its path or interpreter, as the `language-model` change sets.

So the readers contain no extension list.  
`.yml`, `.bash`, `.zsh`, a script that starts `#!/bin/sh` and a `bashrc` file reach YAML and Shell, and no line of this change lists them.  
A fenced code block of the five languages gives its comments through the same dialects, so a ```` ```yaml ```` fence in a Markdown file reads its `#` comments.

Alternative: an extension table for the new readers.  
It lost because langdex already contains every name, extension, file name and interpreter, and a second table drifts from it.

[comments]: https://github.com/victor-software-house/hakkiri/blob/962f38348478048ad7e3f5b0464a8bfb29224f44/src/comments.rs
[code-span-size-6]: ../../code-span-size/design.md#6-vale-drops-an-alert-that-has-the-place-of-another
[v-names]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/code/lang.go#L59
[v-lang]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/code/lang.go#L122
[v-formats]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/format.go#L157
[v-dispatch]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/lint.go#L416
[v-data]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/code_ts.go#L242
[v-directive]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/code/doc.go#L229
[v-shebang]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/code/comments.go#L231
[v-masked]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/code/doc.go#L191
[v-docname]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/code/doc.go#L160
[v-godoc]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/code/doc.go#L77
[v-append]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/code/comments.go#L67
