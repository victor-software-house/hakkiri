# Proposal

Queue row: [`HKR-056`][ledger].

## Motivation

A code span reads as the one-letter word `x` in the text of a sentence, and the tagger tags that word like any other.
A `sequence` rule that reads a noun before a verb therefore flags `` `flag` says ``, where a code name takes the place of a noun.
ProseStyle's `ThingsSpeakNoun` does, and `NounStack` counts the word as a noun of a stack.
Vale 3.26.0 reads a code span as a run of `*` and gives the same alert, so both tools flag code as if it were prose.
Every other check of hakkiri gives no alert on a code span, since code is no prose.

## Changes

- A word that reads a code span is no word that a token can name, by its tag or its pattern.
- A negated token holds at such a word, as it holds at the edge of a sentence.
- A token that reads one word reads the noun that a preposition before the span attaches it to, so `` the notice for `cmd-7` says `` flags `notice`, and `` as `flag` says `` flags nothing.
- The run then starts at that noun, and its message reads that noun.
- The rule applies to every `sequence` rule, because the walk reads each word through one function.
- The `sequence-check` and `vale-differences` specs state it.

## Capabilities

### Modified Capabilities

- `sequence-check`: a code span is no word of a run.
- `vale-differences`: Vale reads the mask of a code span as a word.

## Impact

- Changed: `src/sequence.rs`, `src/check.rs`, the tests of the sequence walk, the instructions, the skill and `AGENTS.md`.
- On nine repositories the change removes 72 alerts of `NounStack` and `ThingsSpeakNoun` and adds 17, and the design lists the true alerts it loses.

[ledger]: ../../../tasks.yaml
