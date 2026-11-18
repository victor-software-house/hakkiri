# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The corrections

- [x] 1.1 Add `src/tagger/corrections.rs` and apply it in `Tagger::tag_words`.
  `Tagger::tag_treebank` keeps the model alone.
  Verify a test for each correction fails on the source before the change and passes after it.
  - 2026-11-12 (UTC): in a box, the tests of the tagger module pass.
    So does `a_verb_after_a_noun_phrase_ends_a_noun_stack` in the sequence module.
  - With the corrections turned off, nine of them fail.
    They are the modal, the name in capitals, the semicolon, the two possessives, the three verb cases and the title.
    Two tests pass either way: a function word, or a letter in capitals, keeps the model's tag.
  - The Vale comparison of the sequence walk reads the tags of the model alone through `find_tagged`.
    It contains rows that Vale gave with that model.
- [x] 1.2 Measure the corrected tags on the Georgetown University Multilayer (GUM) corpus of Universal Dependencies (UD) English.
  Also measure the 200 judged conflicts and the alerts of the nine repositories that run Vale.
  Verify GUM is at least 92.16%, the judged conflicts match or beat the old model, and no removed alert is true.
  - 2026-11-12 (UTC): on the 56,516 tokens of GUM, the old model is right 90.26% of the time.
    The model alone is right 92.16% of the time, and the corrected tags 92.53%.
  - Of the 200 judged conflicts, the old model is right in 131, the model alone in 103 and the corrected tags in 140.
    These counts take `MD` as right for a modal in capitals.
  - `NounStack` and `ThingsSpeakNoun` give 68 alerts fewer under hakkiri and none more.
    All 68 are false when judged by hand.
  - For `NounStack`, we took 100 alerts that only Vale gives and 100 that only hakkiri gives.
    By the text of the rule, four lowercase nouns, 3 and 10 are true.
    If a capitalised noun counts, 15 and 10 are true, because the pattern of the rule has no anchor, as row `HKR-057` records.
  - For `ThingsSpeakNoun`, 18 alerts come only from Vale and 80 only from hakkiri.
    Of those, 4 and 43 are true for a noun of the prose.
    Row `HKR-056` records the code spans that the rest read as nouns.

## 2. The surfaces

- [x] 2.1 Add the `vale-differences` requirements and a patch changeset.
  Add the notes in the instructions, the skill, `AGENTS.md` and the README.
  Verify the OpenSpec validation, `lint:md` and `lint:prose` pass.
  - 2026-11-12 (UTC): these pass in a box: `specs`, `specs:budget`, `lint:md`, `lint:prose` with 0 errors, `lint:typos`, `check:repo` and `check:versions`.
    `format`, `lint`, `test`, `doc`, `deny` and `deps` pass too.
    Not run: `semver`, the advisory check of `deny`, and the workflow policy, which the pull request's CI runs.

## 3. Integration

- [x] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
  Evidence, 2026-11-14 (UTC): The CI of the stack that [pull request 121](https://github.com/victor-software-house/hakkiri/pull/121) heads passed, and the stack merged as `7aac10a`.
