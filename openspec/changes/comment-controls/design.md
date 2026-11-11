# Design

## Context

Vale renders a Markdown file to HTML and walks its tokens.
A comment token that reads `vale off` or `vale Style.Rule = NO` flips a toggle, and the toggles gate each block when the walk reaches its closing tag ([ast.go][v-walk], [file.go][v-file]).
hakkiri has no HTML token stream for Markdown: it parses to a tree whose nodes hold byte ranges of the file.

## Goals / Non-Goals

Goals:

- The alerts match those of Vale on the probe files, in Markdown text and in HTML blocks.
- The forms of hakkiri reach where the forms of Vale reach.

Non-goals:

- Comments of an MDX file or of a source file, and the other markup formats Vale reads.

## Decisions

### 1. A comment acts at two moments, as in Vale

The probe of Vale at `66bbb85` (2026-11-11) over nine files and 304 alerts found these:

- The text after `vale off` is left out of every block until `vale on`, so a rule that reads across the gap reads `start  end`. An unclosed `off` runs to the end of the file.
- A rule is gated per block by the toggles when the element of the block ends. A lone comment in the middle of a paragraph, a list item, a table cell or a heading turns the rule off for the whole element, text before it included.
- An alert between the comment that turned a rule off and the one that turned it on is also dropped, so a heading that holds the comment that turns the rule back on draws no alert before it.
- A tight list item reads when the item ends, so a comment indented under its text reaches the text. A comment in a block of its own, such as the one in a quote, reaches only what follows.
- Inside a `div`, text before an `off` comment keeps its alerts, because Vale reads each run of text as its own block.
- A `raw` rule reads the file at the end, and only `off` gates it. Its alerts between the comments are dropped like the others.
- A comment inside a code block, a code span, an indented code block, or after a backslash reads as text, and so does a comment written in capitals or with another word in front.
- Spaces inside `<!--` and `-->`, line breaks around the text, and none at all are trimmed. `Team.Words=NO` with no spaces around the sign is no comment, and a rule name that no rule has changes nothing.

hakkiri keeps the two moments apart.
`cut` leaves the text between `off` and `on` out of the tree, as Vale leaves it out of the block.
The engine skips a block that the toggles gate when its element ends, and drops an alert that a region holds.
A node holds the byte where its element ends for that: a paragraph in a tight list item ends with the item, and the other nodes end with their own element.

Alternative: drop every alert whose place lies between the comments.
It lost because Vale also gates the text before a lone comment in the same element, and the probe shows both.

### 2. Comments come from the parser

hakkiri reads each comment from the HTML nodes of the Markdown parser, with the tokenizer it already has for HTML blocks, so a comment in a code block or a code span is no comment.
A comment is the text between `<!--` and its end, without the blanks at its ends.
The comment of an HTML block inside a block quote or a list is found in the source text of the node, whose container markers do not touch the comment itself.

### 3. A style comment reads the styles of the rules

`vale styles = Other` turns off the styles the file is based on, except `Other`.
hakkiri has no list of base styles for a file at that point, so it reads the styles of the rules the config runs on the file, which are the same ones.

### 4. Only a Markdown file holds comments that turn rules off

Vale reads a source file's comment as plain lines unless a View names a markup format ([lintComment][v-comment]), so an HTML comment in a doc comment turns nothing off there.
hakkiri reads those comments as Markdown, and turns nothing off either, to match.
Vale reads `{/* vale off */}` in an MDX file, which hakkiri does not yet.

### 5. hakkiri's forms are the config's words

`hakkiri off` and `on`, `hakkiri Style.Rule: off` and `on`, and `hakkiri Style: off` and `on` act as the Vale forms act.
`hakkiri Style.Rule: suggestion`, `warning` or `error` is new: it sets the level of the alerts of a rule from the comment on, until a later comment names the rule, and `on` returns the level of the config.
A level in a comment wins over a level that the config sets, since the comment is closer to the text.
A word that is not one of these is no comment, as in Vale, so a typo is silent.

Alternative: a level for a whole style.
It lost because the config sets levels per rule only.

[v-walk]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/ast.go#L242
[v-file]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/file.go#L758
[v-comment]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/code_ts.go#L199
