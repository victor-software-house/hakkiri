# Proposal

Queue row: `HKR-079` in [tasks.yaml][ledger].

## Why

oxlint reads switches such as `// oxlint-disable-next-line no-console -- reason` in TypeScript and JavaScript, and a person skips them.
hakkiri drops the `eslint` form, as Vale does, but reads the `oxlint` form as prose.
ProseStyle then flags its hyphens and its ` -- ` separator, which oxlint needs as written.
The shared template renders three such lines.

## What Changes

- A line or block comment of TypeScript or JavaScript that opens with `oxlint` is a tool directive.
  hakkiri leaves it out of the tree, as it does an `eslint` switch.
- The `vale-differences` spec records it, since Vale reads it as prose.

## Capabilities

### Modified Capabilities

- `document-tree`: the script directives include `oxlint`.
- `vale-differences`: an `oxlint` switch is a comment that a tool reads.

## Impact

- Changed: `src/comments.rs`, `tests/comment_languages.rs`.
- No new dependency.

[ledger]: ../../../tasks.yaml
