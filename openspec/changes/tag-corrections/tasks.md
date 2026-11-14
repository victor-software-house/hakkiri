# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The corrections

- [x] 1.1 Add `src/tagger/corrections.rs` and apply it in `Tagger::tag_words`, with `Tagger::tag_treebank` on the model alone.
  Verify a test for each correction fails on the source before the change and passes after it.
  Evidence, 2026-11-12 (UTC): in a box the tests of the tagger module and `a_verb_after_a_noun_phrase_ends_a_noun_stack` in the sequence module pass.
  With the corrections turned off, nine of them fail: the modal, the name in capitals, the semicolon, the two possessives, the three verb cases and the title.
  The two tests that a function word or a letter in capitals keeps the model's tag pass either way.
  The Vale comparison of the sequence walk reads the tags of the model alone through `find_tagged`, since it holds rows that Vale gave with that model.
- [x] 1.2 Measure the corrected tags on UD English GUM, on the 200 judged conflicts and on the alerts of the nine repositories that run Vale.
  Verify GUM is at least 92.16%, the judged conflicts are right at least as often as the old model, and no removed alert is true.
  Evidence, 2026-11-12 (UTC): on the 56,516 tokens of GUM the old model is right 90.26% of the time, the model alone 92.16% and the corrected tags 92.53%.
  Of the 200 judged conflicts the old model is right in 131, the model alone in 103 and the corrected tags in 140, counting `MD` as right for a modal in capitals.
  `NounStack` and `ThingsSpeakNoun` give 68 alerts fewer under hakkiri and none more, and all 68 are false by hand.
  Of 100 alerts that only Vale gives and 100 that only hakkiri gives for `NounStack`, 3 and 10 are true by the text of the rule, four lowercase nouns.
  If a capitalised noun counts, 15 and 10 are true, because the pattern of the rule has no anchor, as row `HKR-057` records.
  Of the 18 and 80 that only Vale and only hakkiri give for `ThingsSpeakNoun`, 4 and 43 are true for a noun of the prose, and row `HKR-056` records the code spans the rest read as nouns.

## 2. The surfaces

- [x] 2.1 Add the `vale-differences` requirements, the notes in the instructions, the skill, `AGENTS.md` and the README, and a patch changeset.
  Verify the OpenSpec validation, `lint:md` and `lint:prose` pass.
  Evidence, 2026-11-12 (UTC): `specs`, `specs:budget`, `lint:md`, `lint:prose` with 0 errors, `lint:typos`, `check:repo` and `check:versions` pass in a box, with `format`, `lint`, `test`, `doc`, `deny` and `deps`.
  Not run: `semver`, the advisory check of `deny`, and the workflow policy, which the pull request's CI runs.

## 3. Integration

- [x] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.  
  Evidence, 2026-11-14 (UTC): The CI of the stack that [pull request 121](https://github.com/victor-software-house/hakkiri/pull/121) heads passed, and the stack merged as `7aac10a`.
