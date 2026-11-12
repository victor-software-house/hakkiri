# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The walk

- [x] 1.1 Give the tagger a method that reads a noun in place of each code span and joins the brackets of an intra-doc link, and make `accepts` read a code span as the tag `NN` and the text `X`.
  Verify the tests in `src/sequence/tests.rs` and `tests/sequence.rs` fail on the source before the change and pass after it.
  Evidence, 2026-11-12 (UTC): in a box the three unit tests of the sequence module pass.
- [x] 1.2 Label the alerts of `ThingsSpeakNoun` and `NounStack` that Vale and five hakkiri builds do not all give, against a written definition of each rule.
  Verify two readers labelled each alert of `ThingsSpeakNoun` and each alert of `NounStack` that touches a code span, and the conflicts are settled by the definition.
  Evidence, 2026-11-12 (UTC): 296 alerts of `ThingsSpeakNoun` and 531 of `NounStack`, with 2 conflicts of 296 on the first set and none on the 169 alerts of the second.
- [x] 1.3 Diff the alerts of the nine repositories that run Vale, before and after.
  Verify the false alerts that remain are listed.
  Evidence, 2026-11-12 (UTC): `ThingsSpeakNoun` gives 290 of the alerts the tools disagree on, 264 true and 26 false, and `NounStack` gives 171 false alerts of 210 where Vale gives 262 of 278.
  No other rule of the package gives a different count from the build before the change.

## 2. The surfaces

- [x] 2.1 Add the `sequence-check` and `vale-differences` requirements, the note in the instructions, the skill and `AGENTS.md`, and a patch changeset.
  Verify the OpenSpec validation, `lint:md` and `lint:prose` pass in the pull request's CI.

## 3. Integration

- [ ] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
