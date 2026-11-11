# Proposal

Queue row: `HKR-042` in [tasks.yaml][ledger].
It builds on [vocabularies][vocabularies], which reads the lists, and is blocked by `HKR-019`.

## Motivation

Vale builds two rules from a vocabulary: `Vale.Terms` reports an accepted term written in another case, and `Vale.Avoid` reports a rejected term.
Once hakkiri reads vocabularies, a project that lists `Vale` in its styles gets neither.
The [vale-differences spec][differences] lists the missing rules as a known gap.

## Changes

- hakkiri builds `Vale.Terms` from the accepted terms and `Vale.Avoid` from the rejected ones, with Vale's messages, levels and boundaries.
- A built-in style `Vale` holds the two rules, so `styles: [Vale]` runs them and `Vale.Terms: warning` sets one's level.
- Each vocabulary a glob names builds `Vale.<Name>.Terms` and `Vale.<Name>.Avoid`, which run on that glob's files with no style or setting.
- Without a vocabulary term, the style `Vale` is not a style in any package.
- The `vale-differences` spec drops the gap for the vocabulary rules.

## Capabilities

### New Capabilities

- `vocabulary-rules`: the two rules built from a vocabulary and where they run.

### Modified Capabilities

- `project-config`: a glob may list the built-in style `Vale`.
- `vale-differences`: the gap for the vocabulary rules closes, and so does the gap for a glob's rules.

## Impact

- New: `src/vocabulary_rules.rs`, `tests/vocabulary_rules.rs` and its fixtures, with Vale's alerts recorded from Vale at `66bbb85`.
- Changed: the rule builders, the config loader, `src/instructions.md`, the skill, `AGENTS.md` and `README.md`.
- No new dependency.

[ledger]: ../../../../tasks.yaml
[vocabularies]: ../2026-11-11-vocabularies/proposal.md
[differences]: ../../../specs/vale-differences/spec.md
