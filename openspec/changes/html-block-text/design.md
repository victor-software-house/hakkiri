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

## Goals / Non-Goals

**Goals:**

- The text of an HTML block gets the alerts Vale gives it, with the scope Vale gives it.
- Entities, the text of `code`, `tt`, `kbd` and `samp`, and `pre` match Vale on probes, or a departure records why not.
- The text after a closed raw-text tag on the same line, and after `--!>`, is read.

**Non-Goals:**

- Inline HTML in a paragraph, such as `a<br>b`. It stays a gap, and [HKR-027][hkr-027] measures it.
- Unescaping entities in any text. Vale does it in Markdown text too, and hakkiri does not.

## Decisions

### 1. One walk follows Vale's

`src/html.rs` ports the tokenizer, and a walker follows the part of `lintHTMLTokens` that decides what a block of text is ([ast.go][v-walk]).  
It flushes the text at a start tag that is not inline or void, at an end tag that is not inline, and at the end of the block.  
A flush at a start tag resets the open tags.  
An end tag resets them always.  
The check for a reset reads the same walk, so the two cannot diverge.

Alternative: scan the text with a second, simpler reader.  
Lost: two readers of one tokenizer diverge, and the first already differed on raw-text tags.

### 2. The scope is the outermost element, as Vale reads it

[`lintScope`][v-scope] takes the first tag in the history that is `li`, `blockquote`, `th`, `td` or a heading, so the outermost one wins.  
A list item that holds an HTML block gives its first text the scope `list`, since the tag of the item is still open.  
The tree models that with the `first` flag.  
The first block of text takes the flag of the HTML block, and the later ones do not.  
An `li` or a `blockquote` inside the block becomes a container node over one paragraph, as in Markdown.

Alternative: give every block of text its own scope from its own tags only.  
Lost: `- <div>text</div>` would read as a paragraph, and Vale reads it as `list`.

### 3. What is left out follows Vale's lists

The text of `script`, `style`, `pre`, `noscript` and `iframe` is left out ([ast.go][v-skip]), and so is the text of an element with the class `code`, `pre` or `problematic`.  
Vale masks the first text after `tt`, `code`, `kbd` or `samp` with asterisks, and reads a `code` or `tt` capture as the scope `code`.  
hakkiri reads the text of `code` and `tt` as a code span, which a rule reads as one word ([format 7][format-7]), and leaves out the text of `kbd` and `samp`.  
Vale gives `kbd` no scope of its own ([ast.go][v-inline]), so a code span for it would add alerts.

Alternative: mask with asterisks as Vale does.  
Lost: the tree holds ranges of the file and no replacement text, and a code span already does this job.

### 4. Entities and inline tags read as Markdown text does

Vale unescapes an entity twice and pads the text of an inline element with a space.  
hakkiri reads the bytes of the file.  
So a probe of `marker &amp; marker` finds the match in Vale and not in hakkiri, in an HTML block and in a Markdown paragraph alike.  
`a<br>marker<br/>b` reads as `amarkerb` for the same reason.  
Both are intended differences of the whole Markdown reader.  
They are listed in `vale-differences`, and they are not new in this change.

Alternative: unescape in the text of an HTML block only.  
Lost: the same text would read in two ways, and an alert range over an entity would need a second offset for its end.

### 5. A caption, a description and a data element are not read

Vale gives `caption`, `figcaption`, `data` and `meta` content scopes of their own: `text.table.caption`, `text.figure.caption`, `meta` and `text.attr.description`.  
hakkiri has no node scope for them.  
Reading them as paragraphs would add alerts to the wrong rules.  
So the text is not read, and each case is a scenario of `vale-differences`.

Alternative: add scopes for them.  
Lost: a public enum would change for four elements that few Markdown files hold.

### 6. A block that does not map back is not read

The text of an HTML block loses the markers of the quotes and list items around it, so hakkiri maps each line of it back to the file by its end.  
A line that does not end as the file's does gives no node.

Alternative: tokenize the span of the file.  
Lost: the `>` of a quote would read as text.

### Not ported

- Vale's escaped states for the content of `script`: it ends at the first `</script`.
- The promotion of a block element inside an inline one, and the `vale off` directive.
- Selections and the class scopes, which hakkiri has no rule syntax for.

## Risks / Trade-offs

- [A block reads under the wrong scope] → 31 probe inputs hold lists, quotes, headings and tables, and the parity test fails on any alert that differs and is not listed.
- [The walk differs from Vale on input the probe lacks] → each flag of the walker mirrors a variable of Vale's, and the rows of `departures.tsv` are the only differences found.

[v-walk]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/ast.go#L60-L376
[v-scope]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/ast.go#L378-L420
[v-inline]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/ast.go#L41-L50
[v-skip]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/ast.go#L14-L22
[reset]: ../../../src/html.rs
[case-9]: ../archive/2026-11-10-case-and-sequence-checks/design.md#9-a-block-of-text-has-the-scope-of-one-element
[format-7]: ../archive/2026-11-10-rule-format/design.md#7-intended-differences-from-vale
[hkr-027]: ../../../tasks.yaml
