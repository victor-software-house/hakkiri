# Proposal

Queue row: [`HKR-056`][ledger].

## Motivation

A code span reads as the one-letter word `x` in the text of a sentence, and the tagger tags that word like any other.
Next to a determiner the model tags `x` as a symbol, so a rule of a noun and a verb sees `` `flag` says `` for some spans and not for others.
`NounStack` counts the word `x` as a lowercase noun of a stack, which no reader does.
Two readers labelled every alert of `ThingsSpeakNoun` and `NounStack` that Vale and hakkiri disagree on, against a written definition of each rule.
By that definition a code name before `says` is a thing that speaks, and a code span is never one of the four nouns of a stack.

## Changes

- The tagger reads a noun in place of a code span, so the words around it keep the tags they have beside a noun, and the span has the tag `NN`.
- A pattern or a word list reads a code span as the capital letter `X`: a pattern for any letter takes it, and a pattern for a lowercase word refuses it.
- The square brackets of an intra-doc link, as in ``[`Name`]``, belong to the code span.
- The rule applies to every `sequence` rule, because the walk reads each word through one function.
- The `sequence-check` and `vale-differences` specs state it.

## Capabilities

### Modified Capabilities

- `sequence-check`: a code span is a noun that reads as one capital letter.
- `vale-differences`: Vale reads the mask of a code span as a run of `*`.

## Impact

- Changed: `src/sequence.rs`, `src/tagger.rs`, `src/tagger/tokens.rs`, the tests of the sequence walk, the instructions, the skill and `AGENTS.md`.
- On nine repositories `ThingsSpeakNoun` gives 240 more alerts than the build that refuses the span, 220 true and 20 false by the labels.
  `NounStack` gives 41 fewer than the build before the refusal, all false.
  The design lists the false alerts that remain.

[ledger]: ../../../tasks.yaml
