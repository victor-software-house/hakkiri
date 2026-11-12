# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The walk

- [x] 1.1 Give the tagger a method that reads a noun in place of each code span and joins the brackets of an intra-doc link.
  Make `accepts` read a code span as the tag `NN` and the text `X`.
  Verify the tests in `src/sequence/tests.rs` and `tests/sequence.rs` fail on the source before the change and pass after it.
  Evidence, 2026-11-12 (UTC): in a box the three unit tests of the sequence module pass.
- [x] 1.2 Label the alerts of `ThingsSpeakNoun` and `NounStack` that Vale and the hakkiri builds do not all give, against a written definition of each rule.
  Verify two readers labelled each alert of `ThingsSpeakNoun` and each alert of `NounStack` that touches a code span, a third labelled the alerts only the later builds give, and the conflicts are settled by the definition.
  Evidence, 2026-11-12 (UTC): 304 alerts of `ThingsSpeakNoun` and 540 of `NounStack`, with 2 conflicts of 296 on the first set, none on the 169 alerts of the second, and 1 of 18 on the last.
- [x] 1.3 Diff the alerts of the nine repositories that run Vale, before and after.
  Verify the false alerts that remain are listed.
  Evidence, 2026-11-12 (UTC): `ThingsSpeakNoun` gives 313 of the alerts the tools diverge on, 305 true and 8 false, and every true alert of Vale.
  `NounStack` gives 196 false alerts of 240 where Vale gives 262 of 278, and 44 true where Vale gives 16.
  Two readers labelled each alert, and the design lists the counts of every state and the false alerts that remain.

## 2. The surfaces

- [x] 2.1 Add the `sequence-check` and `vale-differences` requirements, the note in the instructions, the skill and `AGENTS.md`, and a patch changeset.
  Verify the OpenSpec validation, `lint:md` and `lint:prose` pass in the pull request's CI.

## 3. The tagger

- [x] 3.1 Add the corrections of the tagger for the verbs of saying and asking to `src/tagger/corrections.rs`, for the verb tag that the package puts on the verb token of `ThingsSpeakNoun`.
  Verify a unit test pins each correction.
  Evidence, 2026-11-12 (UTC): in a box `cargo test` passes, and `clippy --lib --bins --tests -D warnings` passes.
- [x] 3.2 Measure the tagger on UD English GUM and on the 200 judged conflicts of our documents.
  Verify GUM stays at 92.53% or above.
  Evidence, 2026-11-12 (UTC): GUM is right on 52,298 of 56,516 tokens, 92.54%, against 52,294 before; the judged conflicts count 133 right before and after.

## 4. The pattern check and the technical nouns

- [x] 4.1 Make `run_existence` read a code span as one `*` and drop a match only when it starts or ends on a span.
  Verify the integration test of a match across a span in `tests/alerts.rs` fails on the source before the change and passes after it.
  Verify the test of an edge on a span passes, and the departures of `tests/fixtures/html-block` list the two rows the change adds.
  Evidence, 2026-11-12 (UTC): in a box `cargo test --no-fail-fast` passes and `clippy --lib --bins --tests -D warnings` passes.
  `ClauseChain` gives 1,536 alerts where it gave 1,020, and Vale gives 1,492, with none that only Vale gives.
- [x] 4.2 Add `nouns_of_technical_prose` to `src/tagger/corrections.rs`, with one word list for each place a noun is certain.
  Verify tagger tests pin each list, GUM stays at 92.53% or above, and the 7 true alerts of Vale that `NounStack` lacked are given.
  Evidence, 2026-11-12 (UTC): GUM is right on 52,298 of 56,516 tokens, 92.54%, as before the change, and the judged conflicts count 134 right against 133.
  `NounStack` gives all 16 true alerts of Vale, 14 of them with a span that runs through a link target.
- [x] 4.3 Keep in each list only the words whose sampled readings in that place are all nouns, and put the counts in the design.
  Verify two readers labelled 530 places of the six words in the nine repositories, and each case and list was run alone.
  Evidence, 2026-11-12 (UTC): after a determiner `check`, `prompt`, `thinking` and `fade` are nouns in 163 of 163 places, `round` in 41 of 50 and `split` in 40 of 50, so the last two left that list.
  The title case holds 5 true alerts of `NounStack` and the compound case 1, and the same cases for every word add 98 alerts of which a sample are false.
  GUM, the judged conflicts and every alert of the nine repositories are the same with the three lists.
- [x] 4.4 Correct the conversion of the one repository whose configuration named a dropped style and mapped Rust files to Markdown, and rerun the nine repositories.
  Verify hakkiri gives the 13 alerts of `ClauseChain` that the run lacked for it, and the 40 new alerts of the two other rules are labelled.
  Evidence, 2026-11-12 (UTC): `ThingsSpeakNoun` gives 31 true and 1 false of 32 new alerts, and `NounStack` 1 true and 7 false of 8.
  The 11 true alerts of Vale that hakkiri seemed to lack were this repository, so the sentence splitter is not their cause.

## 5. Integration

- [ ] 5.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
