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

- Comments of a source file, and the other markup formats Vale reads.

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
A node holds the byte where its text is read for that, which decision 7 explains.

Alternative: drop every alert whose place lies between the comments.
It lost because Vale also gates the text before a lone comment in the same element, and the probe shows both.

### 2. Comments come from the parser

hakkiri reads each comment from the HTML nodes of the Markdown parser, with the tokenizer it already has for HTML blocks, so a comment in a code block or a code span is no comment.
A comment is the text between `<!--` and its end, without the blanks at its ends.
The comment of an HTML block inside a block quote or a list is found in the source text of the node, whose container markers do not touch the comment itself.

### 3. A style comment reads the styles of the rules

`vale styles = Other` turns off the styles the file is based on, except `Other`.
hakkiri has no list of base styles for a file at that point, so it reads the styles of the rules the config runs on the file, which are the same ones.

### 4. A source file holds no comment control

Vale reads a source file's comment as plain lines unless a View names a markup format ([lintComment][v-comment]), so an HTML comment in a doc comment turns nothing off there.
hakkiri reads those comments as Markdown, and turns nothing off either, to match.

### 5. hakkiri's forms are the config's words

`hakkiri off` and `on`, `hakkiri Style.Rule: off` and `on`, and `hakkiri Style: off` and `on` act as the Vale forms act.
`hakkiri Style.Rule: suggestion`, `warning` or `error` is new: it sets the level of the alerts of a rule from the comment on, until a later comment names the rule, and `on` returns the level of the config.
A level in a comment wins over a level that the config sets, since the comment is closer to the text.
A word that is not one of these is no comment, as in Vale, so a typo is silent.

Alternative: a level for a whole style.
It lost because the config sets levels per rule only.

### 6. An MDX file reads comments in braces

Vale renders a `{/* … */}` comment of an MDX file, as a flow expression or inside a line of text, as an HTML comment ([mdx.go][v-mdx]).
The probe of 2026-11-11 over one MDX file and 46 alerts showed it read as the HTML form is.
The forms probed are `off` and `on`, a rule, an inline comment, a comment in a tight list item and a comment over three lines.
`{ /* vale off */ }` with a space inside the braces and `{/* Vale Off */}` are text.
hakkiri reads the comment from the MDX expression nodes of the parser, and takes the text between `{/*` and `*/}` without the blanks at its ends.
A gap where hakkiri does less than Vale is fixed, so no difference is recorded.

### 7. The text of a block is read where a tag flushes it

The probe of 2026-11-12 over 10 more files and 1,000 alerts found that Vale does not read a tight item's text when the item ends.
It reads the text buffered so far at each start tag that is not inline or void, and at each end tag that is not inline ([ast.go][v-flush]).
So these read where the first of them comes:

- The text of a tight list item, before a nested list, a quote, a code block, a table, a heading or an HTML block that opens a block tag. A comment inside that block does not reach the text.
- The text of an HTML block, at the next such tag inside the block. A comment between the text and that tag reaches the text, so `<div>one<!-- vale off --></div>` reads nothing, and a nested `div` closing after a comment does the same.
- The text that an HTML block leaves buffered when it ends, at the next block that opens such a tag, or at the end of the container. A comment block between them reaches it.

A thematic break renders a void tag, a definition renders nothing, and a comment is no tag, so none of them flush.
The tokenizer of the HTML walk records the byte of the token that flushes each block of text, and the Markdown builder gives the node of a tight item's paragraph and the node of an HTML block's text that byte.

Alternative: keep the end of the element for every node.
It lost because the probe shows alerts that Vale gives and hakkiri hid, and alerts that hakkiri gave and Vale hid, in these three places.

### 8. Where Vale places a comment wrongly, hakkiri places it right

Vale finds the byte of a comment by searching its text from the end of the previous comment, and finds the place of an alert in an inline element by searching the element's text ([walk.go][v-search]).
Two probes show the search fail:

- A multi-line code span such as `` `<!-- vale off`` and `` -->` `` leaves the text `vale off` in the searched text, so the next real `vale off` comment takes the place of the code span, and Vale hides the alerts between them.
- An alert inside a `strong` element that holds a comment gets the column of the words `vale` in the comment, not the column of its match.

hakkiri places both from the parser, and the `vale-differences` spec records them.

[v-flush]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/ast.go#L242-L252
[v-search]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/walk.go#L667-L690
[v-walk]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/ast.go#L242
[v-file]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/file.go#L758
[v-comment]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/code_ts.go#L199
[v-mdx]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/mdx.go#L1117-L1170
