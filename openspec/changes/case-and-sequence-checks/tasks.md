# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
This change starts after the [`pattern-and-readability-checks` change][pattern] merges.

## 1. Open decisions

- [x] 1.1 Record the operator's answer to each of the four open decisions in [design.md][design], moving each into Decisions with the candidates that lost.  
  Add a `style` scenario to the [capitalization spec][capitalization] for the fourth answer.  
  Verify the Open decisions section is empty and `mise run specs:budget` passes.  
  2026-11-10, the operator's answers are recorded as decisions, and the Open decisions section is gone.
- [x] 1.2 Add the crates, ported code and data files the answers chose, with each license notice the data needs.  
  Verify the `deny` and `deps` steps pass in CI, and that each notice file is in the published crate.  
  2026-11-10, the tagger model and the dictionary are in `main` with their notices in `data/tagger/` and `data/dictionary/`.  
  The `deny` and `deps` tasks ran and passed in the [CI run on the merge commit][main-ci].  
  `Cargo.toml` sets no `include` or `exclude`, so the notice files ship with the crate.  
  Not verified: `cargo package --list`, which CI does not run.

## 2. Shared data

- [x] 2.1 Build the tagger and its word tokenizer as the first answer chose.  
  Verify a unit test in CI tags `The file can be read.` with `MD` on `can`, and `Open the file.` with `NN` on `file`.  
  2026-11-10, `can_is_a_modal_in_a_passive` and `file_is_a_noun_after_an_imperative` pass in CI on [PR 33][hakkiri#33], merged with [PR 35][hakkiri#35] in the [run on `main`][main-ci].  
  A 188-sentence comparison against prose's tags passes as well.
- [x] 2.2 Build the Hunspell reader and the built-in dictionary as the second and third answers chose, with `ICONV`, affix flags and affix expansion.  
  Verify a unit test in CI accepts `receive` and `GitHub`, refuses `blorft`, and expands `utilize/DSG` to four forms.  
  2026-11-10, `words_of_the_dictionary_and_names`, `a_word_it_lacks_is_refused` and `an_affix_flag_makes_four_forms` pass in CI on [PR 35][hakkiri#35], and again in the [run on `main`][main-ci].  
  A 9,159-word comparison against Vale's checker passes as well.

## 3. Checks

- [x] 3.1 Build `capitalization`: the ported converters, the agreement count, indicators, exceptions, `$lower`, `$upper` and patterns.  
  Verify every [capitalization scenario][capitalization] passes as a unit test in CI.  
  2026-11-10, the 14 tests of `tests/capitalization.rs` and `converts_and_counts_as_vale_does`, which compares 1,566 rows with Vale, pass in the [CI run on PR 36][pr36-ci].  
  Kody left no open thread on [PR 36][hakkiri#36].
- [x] 3.2 Build `sequence`: token compilation, `skip` and `min`, the anchor walk, `negate`, `target`, `upos`, `exceptions`, and the refusal of `model`.  
  Verify every [sequence scenario][sequence] passes as a unit test in CI.  
  2026-11-10, the tests of `tests/sequence.rs` and `walks_the_tokens_as_vale_does`, which compares 446 alerts with Vale, pass in the [CI run on PR 37][pr37-ci].  
  Kody left one thread on [PR 37][hakkiri#37], about the steps a large `skip` and `min` allocate; a later commit counts them instead.

## 4. Long inputs

- [ ] 4.1 Run `capitalization` and `sequence` over the rule-engine change's 50,000-character line and 60,000-line file.  
  Verify both tests pass in CI with no panic, and record their run time in the evidence note.

## 5. Integration

- [ ] 5.1 Verify the pull request's CI passes: build, lint, tests, docs, deny, prose lint, OpenSpec and the Template check.

## Workflow follow-up

- Archive this change after the pull request merges, then archive `HKR-014` with `qctl close-from-openspec`.

[hakkiri#33]: https://github.com/victor-software-house/hakkiri/pull/33
[hakkiri#35]: https://github.com/victor-software-house/hakkiri/pull/35
[hakkiri#36]: https://github.com/victor-software-house/hakkiri/pull/36
[hakkiri#37]: https://github.com/victor-software-house/hakkiri/pull/37
[pr36-ci]: https://github.com/victor-software-house/hakkiri/actions/runs/42868740193
[pr37-ci]: https://github.com/victor-software-house/hakkiri/actions/runs/42874900849
[main-ci]: https://github.com/victor-software-house/hakkiri/actions/runs/42865681901
[pattern]: ../archive/2026-11-10-pattern-and-readability-checks/proposal.md
[design]: design.md
[capitalization]: specs/capitalization-check/spec.md
[sequence]: specs/sequence-check/spec.md
