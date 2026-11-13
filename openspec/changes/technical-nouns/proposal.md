# Proposal

Queue row: `HKR-061` in [tasks.yaml][ledger].
It splits out of `code-span-tokens`, whose design went over the word budget.

## Motivation

The model reads `split`, `round`, `check`, `prompt`, `fade` and `thinking` as another part of speech inside a run of nouns, as in `the host load split pull request`.
A run of four nouns lost one, and `NounStack` lost 7 true alerts of Vale.

## Changes

- The tagger reads `check`, `prompt`, `thinking` and `fade` as nouns after a determiner, where the model reads a verb, an adverb or an adjective.
- It reads `prompt`, `round` and `split` as nouns before `pull request` in a title, and `check` as a noun that ends a compound.
- The `vale-differences` spec states it.

## Capabilities

### Modified Capabilities

- `vale-differences`: hakkiri reads the nouns of technical prose, where the model reads another part of speech.

## Impact

- Changed: `src/tagger.rs`, `src/tagger/corrections.rs`, `src/tagger/tests.rs` and `AGENTS.md`.
- On nine repositories `NounStack` gives every true alert of Vale, and GUM stays at 92.54%.
  The design lists the counts of each word list.

[ledger]: ../../../tasks.yaml
