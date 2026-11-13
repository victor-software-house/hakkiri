# Proposal

Queue row: [`HKR-052`][ledger].

## Motivation

hakkiri tags with the English model of prose at `b33e60d`, trained on UD English EWT.
Vale 3.26.0 ships the older model of prose v3.2.2.
On the Markdown of our own repositories the two models give a different tag for 18.4% of the tokens.
Of 200 conflicts drawn at random from our documents and judged by hand, the old model was right in 131 and the new one in 103, with the capital modals counted right as `MD`.
On UD English GUM, a corpus of general English, the new model scores 92.16% and the old one 90.26%.
Both facts hold, so hakkiri keeps the model that scores better in general and corrects the cases that repeat in our prose.
ProseStyle's `NounStack` and `ThingsSpeakNoun` read tags, and a verb read as a noun gives a false alert, as in `The freshness harness reads stdout.`.

## Changes

- A possessive `'s` after a noun reads `POS`.
- A modal written in capitals, `SHALL`, `MUST`, `MAY`, `SHOULD`, `WILL` or `CAN`, reads `MD`.
- A token of two or more capital letters reads `NNP`, unless it is `KB`, `MB`, `GB`, `TB` or `PB`, or the model gives it the tag of a function word.
- A semicolon reads `:`.
- A plural noun after a noun reads `VBZ` before an object, and before a noun, an adjective or a preposition other than `of` when no other word of the sentence is a verb.
- `Tagger::tag` and `Tagger::tag_words` apply the rules.
  `Tagger::tag_treebank` does not, so a title is tagged as prose tags it.
- The `vale-differences` spec lists the corrections with the numbers above.

## Capabilities

### Modified Capabilities

- `vale-differences`: the tags of the corrected model.

## Impact

- New: `src/tagger/corrections.rs`.
- Changed: `src/tagger.rs`, `src/tagger/tests.rs`, `src/instructions.md`, `skills/hakkiri/SKILL.md`, `AGENTS.md` and a patch changeset.
- Not changed: the model file, the tokenizer, and the capitalization check.

[ledger]: ../../../tasks.yaml
