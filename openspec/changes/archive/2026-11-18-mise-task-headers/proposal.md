# Proposal

Queue row: `HKR-077` in [tasks.yaml][ledger].

## Why

A mise file task opens with header lines such as `#MISE description="Build the docs"` and `#USAGE arg "<file>"`.  
mise reads them, and a person reads the plain comments after them.  
hakkiri reads them as prose, so ProseStyle flags `MISE` as an acronym with no definition, once per task.  
A render of the shared template has 47 such alerts.

## What Changes

- A Shell comment that opens with `#MISE`, `#USAGE`, `# [MISE]` or `# [USAGE]` is a tool directive.  
  A bare keyword counts only right after `#`, so `# USAGE deploy.sh <env>` stays prose.  
  hakkiri leaves it out of the tree, as it does `# shellcheck disable=SC2086`.
- The instructions list these headers beside the other directives.

## Capabilities

### Modified Capabilities

- `document-tree`: the Shell directives include the mise task headers.

## Impact

- Changed: `src/comments.rs`, `src/instructions.md`, `tests/fixtures/comment-languages`.
- No new dependency.

[ledger]: ../../../../tasks.yaml
