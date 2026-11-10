# Proposal

Queue row: `HKR-003` in [tasks.yaml][ledger].

## Motivation

A rule selects text by scope ([rule-format design][rule-format]).  
So every file hakkiri reads must become one tree whose nodes carry those scope names and their byte spans.  
The engine slice `HKR-004` runs checks over this tree, and alerts point back into the file through the spans.

## Changes

- Markdown and MDX files parse into the tree, each node named by its Vale scope.
- Rust, TypeScript and JavaScript files give the tree their comments only, each read as Markdown, so string literals never reach it.
- Every node holds the byte span of its text in the file, including text inside a comment.
- Snapshot tests show the tree for block comments, strings that hold URLs, nested lists, tables and MDX.

## Capabilities

### New Capabilities

- `document-tree`: which nodes a file gives, the scope each carries, and its span.

### Modified Capabilities

None.

## Impact

- New dependencies: markdown-rs for Markdown and MDX, tree-sitter with the Rust, TypeScript and JavaScript grammars for comments, and insta for snapshot tests.
- Sentences and words are not in this slice; the engine slice adds them with word counting.

[ledger]: ../../../tasks.yaml
[rule-format]: ../archive/2026-11-10-rule-format/design.md
