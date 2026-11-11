# Proposal

Queue row: `HKR-021` in [tasks.yaml][ledger].

## Motivation

Vale renders a Markdown file to HTML and lints the text of the result.  
So Vale lints the text of an HTML block, with the scope of its block, and hakkiri reads an HTML block as markup and lints none of it.  
A project that moves from Vale to hakkiri loses every alert in that text.

The operator decided that hakkiri reads the text inside HTML blocks as prose, as Vale does (operator, 2026-11-10).  
It is a gap to close, and not a difference to keep.  
The operator also ruled that hakkiri gives at least Vale's result, and departs only where Vale's behaviour is a defect (operator, 2026-11-10).

## Changes

- The text of an HTML block becomes nodes of the tree, with the scope Vale gives it: `paragraph`, `list`, `blockquote`, a heading, `table.header`, `table.cell`, `table.caption`, `figure.caption` or `meta`.  
- Four scopes are new: `table.caption`, `figure.caption`, `meta` and `description`, the content of a `meta` tag that describes the page.
  Four more are the headings of depth 0 and 7 to 9, `heading.h0` and `heading.h7` to `heading.h9`, which Vale reads for an `h0` or an `h7` to `h9` element.
- Alt text is read as `text` too, as Vale reads it, and so is a description.
- Some text is left out: that of `script`, `style`, `noscript`, `iframe` and `pre`, and of an element with the class `code`, `pre` or `problematic`.
- Some text reads as `*` for each character, as in Vale: that of `kbd` and `samp`.
- The text of `code` and `tt` is a code span, and `a`, `strong`, `b`, `em`, `i` and `img` give a link, strong text, emphasis and alt text.
- An inline or void tag joins the text around it as Vale joins it, in an HTML block and in a Markdown paragraph.
- A character reference reads as its character in a paragraph, heading, cell, caption, link, strong text, emphasis, alt text and description, as `golang.org/x/net/html` reads it.
- The text after a closed raw-text tag on the same line, and after `--!>`, is read, and a `<!--` in a `script` hides its end tag as HTML5 says.
- A block element that opens inside an inline one reads as inline.
- Each difference that is left is a scenario of `vale-differences`, probed against Vale `66bbb85` on a fixture.

## Capabilities

### New Capabilities

- `html-block-text`: which text of an HTML block hakkiri reads, under which scope, and how the tags join it.
- `html-tokenizer`: which whitespace ends a tag name and an attribute.

### Modified Capabilities

- `document-tree`: raw HTML tags stay out of the prose, the text between them is prose, and four scopes join the table.
- `node-segments`: a segment can read otherwise than as written.
- `scope-selection`: alt text and captions are `text`, and a character reference reads as its character.
- `vale-differences`: the known gap closes, and each difference it leaves is listed.

## Impact

- `src/html.rs` becomes a port of the tokenizer of `golang.org/x/net/html` and of Vale's walk of its tokens, with the spacing variables of that walk.
  The check for whether an HTML block ends a run of tags reads that walk.
- A new `src/entity.rs` and `data/entities/html5.tsv` read character references.
- A new `src/html_text.rs` maps the walk back to the lines of the file, and the Markdown builder adds the nodes.
- `Node` gains `reads`, a reading for each segment, and `Scope` gains eight variants.
- `tests/html_parity.rs` compares 41 rules over 52 inputs with Vale.

[ledger]: ../../../tasks.yaml
