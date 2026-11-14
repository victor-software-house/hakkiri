# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. Spec and test

- [ ] 1.1 State in the vocabularies spec that a match holding an accepted word as a whole word with its case is held back, in both requirements.  
  Verify `openspec validate --all --strict` passes.
- [ ] 1.2 Add `a_match_that_holds_an_accepted_word_is_held_back` to `tests/vocabularies.rs` with the probe's four sentences.  
  Verify the test passes in CI.

## 2. Integration

- [ ] 2.1 Verify the pull request's CI passes: lint, prose lint, OpenSpec and the tests.
