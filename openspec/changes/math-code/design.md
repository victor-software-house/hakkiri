# Design

## Context

Vale parses math with two parsers of its own ([math.go][v-math]): `$…$` inline and `$$…$$` for a block.
markdown-rs has math constructs that read a `$` pair whatever stands around it, so `$5 and $10` is math to it.
hakkiri parses without them.

## Goals / Non-Goals

Goals:

- The alerts match those of Vale on the probe files, in Markdown and MDX.
- Every place in the tree is a place in the file.

Non-goals:

- Math in a comment of a source file, since Vale reads a comment as plain lines and finds none.

## Decisions

### 1. The file is parsed, masked and parsed again

hakkiri finds the math in the text of each paragraph, heading and table cell of a first parse, and writes the file again with the same length.
An inline span's `$` delimiters become backticks, and a backtick inside it becomes an apostrophe, so it parses as a code span.
Each line of a display block becomes blank, so the paragraph ends before the block and starts again after it.
The second parse gives the tree, and the code node of a span takes the `$` delimiters back into its segments, which Vale's code span holds.

Alternative: enable the math of markdown-rs.
It lost because its rules differ from Vale's on money, wrapped spans and block endings, and the probe shows each.
Alternative: rewrite the text as the ignore patterns do.
It lost because the length changes, and a same-length mask needs no map of places.

### 2. The probe of Vale fixes the rules

The probe of Vale at `66bbb85` (2026-11-11), over three Markdown files, one MDX file and 80 alerts, found these:

- `$5 and $10`, `US$5 and CA$10`, `A lone $5` and `$ very$` are text, and `\$5` is text.
- `$g_i =` and `really$` on the next line are one span.
- `$a$5 very then $b$` is one span, since the first closing `$` is followed by a digit.
- `$$x very$$` inside a line holds the span `$x very$`, because the first `$` cannot open and the second can.
- A span holds `*very*` and a backtick as written, since the math parser reads before the emphasis and code parsers do.
- A display block can end with `$$` after content, with `$$ {#eq-a}`, with a blank line, or with the paragraph, and it can start in the middle of a paragraph, in a list item and in a block quote.
- An MDX file reads the same rules, and skips a `{…}` expression first.
- A `code` rule reads `$a$` with its delimiters, so a rule that matches `$` flags both.

### 3. A container's markers stay

A line after the first of a paragraph in a block quote or a list item holds the container's markers and indentation.
hakkiri skips them when it looks for `$$` at the start of a line, and a masked line keeps them, so a quote or an item goes on.

[v-math]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/math.go
