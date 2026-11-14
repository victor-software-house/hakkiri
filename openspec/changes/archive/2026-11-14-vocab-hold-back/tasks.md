# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. Spec and test

- [x] 1.1 State in the vocabularies spec that a match holding an accepted word as a whole word with its case is held back, in both requirements.  
  Verify `openspec validate --all --strict` passes.  
  Evidence, 2026-11-14 (UTC): the spec delta of this change states the rule in both requirements, and `openspec validate --all --strict` passes.
- [x] 1.2 Add `a_match_that_holds_an_accepted_word_is_held_back` to `tests/vocabularies.rs` with the probe's four sentences.  
  Verify the test passes in CI.  
  Evidence, 2026-11-14 (UTC): the CI of [pull request 166](https://github.com/victor-software-house/hakkiri/pull/166) passed at `f6642cf`, `a_match_that_holds_an_accepted_word_is_held_back` among its tests.

## 2. Integration

- [x] 2.1 Verify the pull request's CI passes: lint, prose lint, OpenSpec and the tests.  
  Evidence, 2026-11-14 (UTC): the CI of [pull request 166](https://github.com/victor-software-house/hakkiri/pull/166) passed at `f6642cf`.
