# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
This change starts after the [`pattern-and-readability-checks` change][pattern] merges.

## 1. Open decisions

- [x] 1.1 Record the operator's answer to each of the four open decisions in [design.md][design], moving each into Decisions with the candidates that lost.  
  Add a `style` scenario to the [capitalization spec][capitalization] for the fourth answer.  
  Verify the Open decisions section is empty and `mise run specs:budget` passes.  
  2026-11-10, the operator's answers are recorded as decisions, and the Open decisions section is gone.
- [ ] 1.2 Add the crates, ported code and data files the answers chose, with each license notice the data needs.  
  Verify the `deny` and `deps` steps pass in CI, and that each notice file is in the published crate.

## 2. Shared data

- [ ] 2.1 Build the tagger and its word tokenizer as the first answer chose.  
  Verify a unit test in CI tags `The file can be read.` with `MD` on `can`, and `Open the file.` with `NN` on `file`.
- [ ] 2.2 Build the Hunspell reader and the built-in dictionary as the second and third answers chose, with `ICONV`, affix flags and affix expansion.  
  Verify a unit test in CI accepts `receive` and `GitHub`, refuses `blorft`, and expands `utilize/DSG` to four forms.

## 3. Checks

- [ ] 3.1 Build `capitalization`: the ported converters, the agreement count, indicators, exceptions, `$lower`, `$upper` and patterns.  
  Verify every [capitalization scenario][capitalization] passes as a unit test in CI.
- [ ] 3.2 Build `sequence`: token compilation, `skip` and `min`, the anchor walk, `negate`, `target`, `upos`, `exceptions`, and the refusal of `model`.  
  Verify every [sequence scenario][sequence] passes as a unit test in CI.
- [ ] 3.3 Build `spelling`: Vale's word tokenizer, the filters, `ignore`, `split`, the two message rewrites, package dictionaries, and the refusal of a path that leaves the package.  
  Verify every [spelling scenario][spelling] passes as a unit test in CI.
- [ ] 3.4 Expand `affix` in `existence` and `substitution`, and remove the rule-engine change's refusal of the field.  
  Verify both affix scenarios pass in CI, and the rule-engine change's `affix` refusal test is gone.

## 4. Long inputs

- [ ] 4.1 Run the three check types over the rule-engine change's 50,000-character line and 60,000-line file.  
  Verify both tests pass in CI with no panic, and record their run time in the evidence note.

## 5. Integration

- [ ] 5.1 Verify the pull request's CI passes: build, lint, tests, docs, deny, prose lint, OpenSpec and the Template check.

## Workflow follow-up

- Archive this change after the pull request merges, then archive `HKR-014` with `qctl close-from-openspec`.

[pattern]: ../archive/2026-11-10-pattern-and-readability-checks/proposal.md
[design]: design.md
[capitalization]: specs/capitalization-check/spec.md
[sequence]: specs/sequence-check/spec.md
[spelling]: specs/spelling-check/spec.md
