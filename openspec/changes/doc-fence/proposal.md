# Proposal

Queue row: `HKR-046` in [tasks.yaml][ledger].

## Motivation

A fenced block in a Rust doc comment stopped being code at a blank line between two `///` lines.
Rustdoc reads the doc comments of one item as one text, so the fence stays open.
hakkiri cut the comment into two nodes at the blank line, and read the lines after it as prose.
The ProseStyle v0.0.14 `BareURLInComment` case `keeps a doc comment's fence open across a blank line` failed for that reason.

## Changes

- Doc line comments of the same type join across blank lines, as rustdoc reads them.
- Plain line comments and block comments join as before.
- The `document-tree` spec states it, and the `vale-differences` spec records that Vale joins consecutive lines only.

## Capabilities

### Modified Capabilities

- `document-tree`: doc line comments join across blank lines.
- `vale-differences`: where a run of doc line comments ends.

## Impact

- Changed: `src/comments.rs`, `tests/alerts.rs`, `tests/test_command.rs`.
- New: `tests/fixtures/bare-url-comment`.
- No new dependency.

[ledger]: ../../../tasks.yaml
