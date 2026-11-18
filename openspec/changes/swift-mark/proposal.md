# Proposal

Queue row: `HKR-080` in [tasks.yaml][ledger].

## Why

Xcode reads `// MARK: - Title` in a Swift file and shows the title as a section in its jump bar.
hakkiri reads the comment as prose, so ProseStyle flags `MARK` as an acronym on every use.
A repository then has to drop its Xcode sections to pass the prose lint.

## What Changes

- A Swift line comment that opens with `MARK:` is a tool directive.
  hakkiri leaves it out of the tree, as it does a `swiftlint:` switch.
- The `vale-differences` spec records it, since Vale reads it as prose.

## Capabilities

### Modified Capabilities

- `document-tree`: the Swift directives include `MARK:`.
- `vale-differences`: a `MARK:` comment is a comment that a tool reads.

## Impact

- Changed: `src/comments.rs`, `tests/comment_languages.rs`.
- No new dependency.

[ledger]: ../../../tasks.yaml
