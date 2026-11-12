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
- The tagger corrects the verbs of saying and asking after a noun, for the verb tag that the package puts on the verb token of `ThingsSpeakNoun`.
- An `existence` rule reads a code span as one `*`, as Vale masks it, and drops a match only when it starts or ends on a span, so a match across a span is a match.
- The tagger reads `check`, `split`, `round`, `prompt`, `fade` and `thinking` as nouns in a noun phrase, where the model reads a verb, an adverb or an adjective.
- The `sequence-check`, `scope-selection` and `vale-differences` specs state it.

## Capabilities

### Modified Capabilities

- `sequence-check`: a code span is a noun that reads as one capital letter.
- `scope-selection`: an `existence` match across a code span is a match.
- `vale-differences`: Vale reads the mask of a code span as a run of `*`, and hakkiri reads the nouns of technical prose.

## Impact

- Changed: `src/sequence.rs`, `src/tagger.rs`, `src/tagger/tokens.rs`, `src/tagger/corrections.rs`, the tests of the sequence walk and the tagger, the instructions, the skill and `AGENTS.md`.
- On nine repositories `ThingsSpeakNoun` gives 281 of the alerts the tools disagree on, 274 true and 7 false, and every true alert of Vale.
  `NounStack` gives 232, 43 true and 189 false where Vale gives 278, 16 true and 262 false, and `ClauseChain` gives 1,523 alerts where it gave 1,020 and Vale gives 1,492.
  GUM stays at 92.54%.
  The design lists the counts of every state and the false alerts that remain.

[ledger]: ../../../tasks.yaml
