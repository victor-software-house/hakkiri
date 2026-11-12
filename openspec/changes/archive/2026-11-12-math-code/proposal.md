# Proposal

Queue row: `HKR-048` in [tasks.yaml][ledger].
It follows [source-comment ignores][ignores] in the stack, and uses nothing from it.

## Motivation

Vale reads `$…$` and `$$…$$` in a Markdown or MDX file as math, and no prose rule reads math: an inline span is a code span, and a display block is a code block.
markdown-rs reads a `$` pair as math by other rules, and hakkiri's parser enables none of its math, so a text rule flagged the words of an equation.
The probe in `tests/fixtures/markup-differences` showed it, and the row `HKR-048` queued it.

## Changes

- An inline span follows Pandoc's delimiters, as Vale does.
  The opening `$` is followed by a character that is not a space, and the closing `$` follows one and is not followed by a digit.
  A span wraps over the lines of a paragraph.
- A display block opens with a line that starts with `$$`, even in the middle of a paragraph, and closes with a line that ends with `$$`, a blank line or the end of the paragraph.
- Both read as code, so text and sentence rules skip them and a `code` rule reads an inline span with its `$` delimiters.
- The difference is gone from the `block-token-ignores` change, which listed it.

## Capabilities

### New Capabilities

- `math`: the spans and blocks that read as math, and what a rule reads in them.
- `mdx-code`: the expressions and childless JSX elements of an MDX file that read as code.

### Modified Capabilities

- `vale-differences`: the difference for math is removed from the change that listed it.

## Impact

- New: `src/math.rs`, `tests/math.rs` and its fixtures, with Vale's alerts recorded from Vale at `66bbb85`.
- Changed: the Markdown builder, `src/instructions.md`, `README.md` and `AGENTS.md`.
- No new dependency.

[ledger]: ../../../../tasks.yaml
[ignores]: ../2026-11-12-block-token-ignores/proposal.md
