# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
This change builds on the dictionary reader and the built-in dictionary that the [`case-and-sequence-checks` change][case] merged.

## 1. Checks

- [ ] 1.1 Build `spelling`: Vale's word tokenizer, the filters, `ignore`, `split`, the two message rewrites, package dictionaries, and the refusal of a path that leaves the package.  
  Verify every [spelling scenario][spelling] passes as a unit test in CI.
- [ ] 1.2 Expand `affix` in `existence` and `substitution`, and remove the rule-engine change's refusal of the field.  
  Verify both affix scenarios pass in CI, and the rule-engine change's `affix` refusal test is gone.

## 2. Long inputs

- [ ] 2.1 Run `spelling`, and `existence` and `substitution` with `affix`, over the rule-engine change's 50,000-character line and 60,000-line file.  
  Verify both tests pass in CI with no panic, and record their run time in the evidence note.

## 3. Integration

- [ ] 3.1 Verify the pull request's CI passes: build, lint, tests, docs, deny, prose lint, OpenSpec and the Template check.

## Workflow follow-up

- Archive this change after the pull request merges, then archive `HKR-020` with `qctl close-from-openspec`.

[case]: ../case-and-sequence-checks/proposal.md
[spelling]: specs/spelling-check/spec.md
