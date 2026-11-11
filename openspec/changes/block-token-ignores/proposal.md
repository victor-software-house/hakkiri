# Proposal

Queue row: `HKR-043` in [tasks.yaml][ledger].
It follows [vocabulary-rules][vocabulary-rules] in the stack and needs nothing from it.

## Motivation

Vale's `BlockIgnores` and `TokenIgnores` hide text from the prose rules.
OpenSpec 1.16 and 1.17 refuse a proposal without the headings `Why` and `What Changes`, and the ProseStyle rule `ChattyHeadings` flags both.
Six repositories skip those two lines with `BlockIgnores` today, and one uses `TokenIgnores`.
A project that moves from Vale to hakkiri has no way to say the same, so it either lints text it hid before or edits its documents.

## Changes

- A `files` glob takes `block_ignores` and `token_ignores`, each a list of patterns, in the config of a project and in a package's settings file.
- In a Markdown or MDX file, each match of a block pattern is read as a fenced code block that holds the first group of the match, as Vale reads it.
- Each match of a token pattern is read as a code span that holds the first group.
- The patterns of every glob that matches a file apply, a package's first and then the project's, so a project adds to a package.
- The Markdown of a source file's comment takes the patterns too, where Vale applies none to a comment it reads as plain lines.
- The differences from Vale go in the `vale-differences` spec, one more that the probe found among them: a sentence rule in a comment.

## Capabilities

### New Capabilities

- `block-token-ignores`: the two keys, what a match turns into, and where it applies.

### Modified Capabilities

- `vale-differences`: the ways the ignores differ from Vale's.

## Impact

- New: `src/ignore.rs`, `tests/ignores.rs` and its fixtures, with Vale's alerts recorded from Vale at `66bbb85`.
- Changed: the glob settings of the config loader, the parse entry point of the library, `src/instructions.md`, the skill, `README.md` and `AGENTS.md`.
- No new dependency.

[ledger]: ../../../tasks.yaml
[vocabulary-rules]: ../archive/2026-11-11-vocabulary-rules/proposal.md
