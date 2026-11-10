# Proposal

Queue row: `HKR-021` in [tasks.yaml][ledger].

## Motivation

Vale renders a Markdown file to HTML and lints the text of the result.  
So Vale lints the text of an HTML block, with the scope of its block, and hakkiri reads an HTML block as markup and lints none of it.  
A project that moves from Vale to hakkiri loses every alert in that text.

The operator decided that hakkiri reads the text inside HTML blocks as prose, as Vale does (operator, 2026-11-10).  
It is a gap to close, and not a difference to keep.

## Changes

- The text of an HTML block becomes nodes of the tree, with the scope Vale gives it: `paragraph`, `list`, `blockquote`, a heading, `table.header` or `table.cell`.
- Some text is left out: that of `script`, `style`, `noscript`, `iframe`, `pre`, `kbd` and `samp`, and of an element with the class `code`, `pre` or `problematic`.
- The text of `code` and `tt` is a code span, and `a`, `strong`, `b`, `em`, `i` and `img` give a link, strong text, emphasis and alt text.
- The text after a closed raw-text tag on the same line, and after `--!>`, is read.
- Each difference that is left is a scenario of `vale-differences`, probed against Vale `66bbb85` on a fixture.

## Capabilities

### New Capabilities

- `html-block-text`: which text of an HTML block hakkiri reads, and under which scope.

### Modified Capabilities

- `document-tree`: raw HTML tags stay out of the prose, and the text between them is prose.
- `vale-differences`: the known gap closes, and each difference it leaves is listed.

## Impact

- `src/html.rs` becomes a port of the tokenizer of `golang.org/x/net/html` and of Vale's walk of its tokens.
  The check for whether an HTML block ends a run of tags reads that walk.
- A new `src/html_text.rs` maps the walk back to the lines of the file, and the Markdown builder adds the nodes.
- `tests/html_parity.rs` compares 31 rules over 31 inputs with Vale.

[ledger]: ../../../tasks.yaml
