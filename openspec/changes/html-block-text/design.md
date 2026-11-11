# Design

## Context

Vale reads a Markdown file in two steps.  
It renders the file to HTML with goldmark.  
Then it walks the tokens of that HTML with the tokenizer of `golang.org/x/net/html` ([ast.go][v-walk]).  
An HTML block passes through the first step as written, so its text is text in the second.

hakkiri reads Markdown with markdown-rs.  
Its tree held each HTML block as a gap.  
[`resets_scope`][reset] already followed the tokenizer for one question.  
The question is whether the block ends the run of tags that gives a block its scope ([case 9][case-9]).

The operator ruled that hakkiri gives at least Vale's result, and departs only where Vale's behaviour is a defect (operator, 2026-11-10).  
So each way Vale reads this text is matched, and each departure has its reason.

## Goals / Non-Goals

**Goals:**

- The text of an HTML block gets the alerts Vale gives it, with the scope Vale gives it.
- A character reference, the text of `code`, `tt`, `kbd` and `samp`, `pre`, the tags between words, captions, descriptions and alt text read as Vale reads them, or a departure records why not.
- The text after a closed raw-text tag on the same line, and after `--!>`, is read.
- A block element inside an inline one, and a `<!--` in a script, read as Vale reads them.

**Non-Goals:**

- Scopes for the elements of inline HTML in a Markdown paragraph, such as `<b>` and `<a>`. The text around the tags reads as Vale reads it, and [HKR-027][hkr-027] measures the rest.
- Markdown's own inline markup as tokens of the walk. Vale pads `a*b*c` as `a bc`, and hakkiri reads `abc`.

## Decisions

### 1. One walk follows Vale's

`src/html.rs` ports the tokenizer, and a walker follows the part of `lintHTMLTokens` that decides what a block of text is ([ast.go][v-walk]).  
It flushes the text at a start tag that is not inline or void, at an end tag that is not inline, and at the end of the block.  
A flush at a start tag resets the open tags.  
An end tag resets them always.  
A block element that opens inside an inline element counts as inline, and so does its end tag.  
The check for a reset reads the same walk, so the two cannot diverge.

Alternative: scan the text with a second, simpler reader.  
Lost: two readers of one tokenizer diverge, and the first already differed on raw-text tags.

### 2. The scope is the outermost element, as Vale reads it

[`lintScope`][v-scope] takes the first tag in the history that is `li`, `blockquote`, `th`, `td`, a heading, `caption` or `figcaption`, so the outermost one wins.  
A `data` element anywhere in the history gives the scope `meta`.  
A list item that holds an HTML block gives its first text the scope `list`, since the tag of the item is still open.  
The tree models that with the `first` flag.  
The first block of text takes the flag of the HTML block, and the later ones do not.  
An `li` or a `blockquote` inside the block becomes a container node over one paragraph, as in Markdown.

Alternative: give every block of text its own scope from its own tags only.  
Lost: `- <div>text</div>` would read as a paragraph, and Vale reads it as `list`.

### 3. Masked text reads as asterisks

Vale masks the first text after a `tt`, `code`, `kbd` or `samp` start tag, the text of an element whose class is `code`, `pre` or `problematic`, and the text of a link that is the link's address.  
A masked text reads as one `*` for each character, and a line feed stays.  
A segment of a node carries this as its [reading][tree-reading].  
The text of `script`, `style`, `pre`, `noscript` and `iframe` is left out ([ast.go][v-skip]).

hakkiri reads `code` and `tt` as a code span, which a rule reads as one word ([format 7][format-7]), and gives it a `code` node as Vale does.  
So only `kbd`, `samp`, a skipped class and an address read as asterisks.  
Vale also leaves a block of only masked text unplaced, and hakkiri alerts on it.

Alternative: leave the text of `kbd` and `samp` out.  
Lost: a rule that matches across a key, such as `press Ctrl then`, reads `press then` and misses what Vale sees.

### 4. Tags join the text around them

Vale pads the text after an inline or void start tag with a space, and keeps the source's separator after an inline end tag or a comment ([`clean`][v-clean]).  
A text loses the whitespace at its ends, and no space follows an opening bracket, a dash or a non-ASCII space.  
`Spacer` in `src/html.rs` holds those variables, and the walker of an HTML block and the Markdown builder both use it.  
Each segment holds the result as its reading, which replaces the separator that the gap would give.

In a Markdown paragraph, only an inline or void tag and a comment change the join.  
A tag that opens a block ends the text in Vale, and hakkiri keeps the text around it as one text, as before.

Alternative: read the gap as a space when it holds a tag.  
Lost: `a<b>x</b>c` reads `a xc` in Vale, and a gap rule gives `a x c`.

### 5. A character reference reads once

`src/entity.rs` ports the unescaping of `golang.org/x/net/html` at v0.62.0.  
It reads named references from its table, and those without a `;` that HTML allows.  
It reads decimal and hexadecimal ones, with the table for `0x80` to `0x9F`, and it has the stricter rule for an attribute value.  
`data/entities/html5.tsv` holds the table of that version.  
A paragraph, heading, cell, caption, link, strong text, emphasis, alt text and description read it, and a code span, a comment and `raw` do not.  
An alert on a character that a reference gave points at the first byte of the reference.  
Vale gives `&` for `&amp;` the same place.

Vale unescapes the text of each token twice, so `&amp;lt;` reads as `<` and a code span `&amp;` reads as `&`.  
That is a defect, and hakkiri unescapes once.  
Vale also places a match by finding the words of the text in the file, so text that holds a reference loses alerts and gets some on the wrong line.

Alternative: use a crate for the table.  
Lost: a dependency for one table, whose entries must equal Vale's version, and a crate may differ in the legacy names.

### 6. Captions, data and descriptions have scopes

Vale gives `caption`, `figcaption`, `data` and a `meta` description the scopes `text.table.caption`, `text.figure.caption`, `meta` and `text.attr.description`, and gives alt text `text.attr.alt` ([ast.go][v-tags]).  
hakkiri adds the scopes `table.caption`, `figure.caption`, `meta` and `description`.  
Alt text, a description and a caption are `text` too, and `meta` is not.  
Alt text and a description are one block each, and no sentence splitter cuts them, as in Vale.  
A heading `h0`, or `h7` to `h9`, reads as the scope `heading.h0` or `heading.h7` to `heading.h9`, since Vale takes any element named `h` and a digit for a heading.

Alternative: keep `alt` apart from `text`.  
Lost: Vale reports a word in alt text for every default rule, and hakkiri would not.

### 7. A block that does not map back is not read

The text of an HTML block loses the markers of the quotes and list items around it, so hakkiri maps each line of it back to the file by its end.  
A line that does not end as the file's does gives no node.

Alternative: tokenize the span of the file.  
Lost: the `>` of a quote would read as text.

### Not ported

- The `vale off` and `vale on` comments (row HKR-031), and the selections and class scopes, which hakkiri has no rule syntax for.
- A text equal to the name of an element of a skipped class, which Vale counts as a tag.

## Risks / Trade-offs

- [A block reads under the wrong scope or with the wrong join] → the probe holds 52 inputs, from lists and quotes to joins, masks, references, scripts and promoted blocks.  
  The parity test fails on any alert that differs and is not listed.
- [The walk differs from Vale on input the probe lacks] → each flag of the walker mirrors a variable of Vale's, and the rows of `departures.tsv` are the only differences found.
- [The tree type grows] → `Node` gains `reads`, and `Scope` gains eight variants. hakkiri is not released, and it owns both types.

[v-walk]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/ast.go#L60-L376
[v-scope]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/ast.go#L378-L420
[v-tags]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/ast.go#L619-L651
[v-clean]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/ast.go#L734-L761
[v-skip]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/ast.go#L14-L22
[reset]: ../../../src/html.rs
[tree-reading]: ../../../src/tree.rs
[case-9]: ../archive/2026-11-10-case-and-sequence-checks/design.md#9-a-block-of-text-has-the-scope-of-one-element
[format-7]: ../archive/2026-11-10-rule-format/design.md#7-intended-differences-from-vale
[hkr-027]: ../../../tasks.yaml
