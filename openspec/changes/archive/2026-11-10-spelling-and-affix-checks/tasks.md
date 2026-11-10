# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
This change builds on the dictionary reader and the built-in dictionary that the [`case-and-sequence-checks` change][case] merged.

## 1. Checks

- [x] 1.1 Build `spelling`: Vale's word tokenizer, the filters, `ignore`, `split`, the two message rewrites, package dictionaries, and the refusal of a path that leaves the package.  
  Verify every [spelling scenario][spelling] passes as a unit test in CI.  
  - 2026-11-10 (UTC), the 18 tests of `tests/spelling.rs` and `spelling_reports_as_vale_does_but_where_decided`, which compares Vale's alerts over the spelling inputs, pass in the [CI run on PR 41][pr41-ci].  
    [PR 44][hakkiri#44] added the refusal of a field with no effect: the 19 tests of `tests/spelling.rs` pass in the [CI run on PR 46][pr46-ci].  
    Kody left two threads on [PR 41][hakkiri#41], both fixed in PR 44.
- [x] 1.2 Expand `affix` in `existence` and `substitution`, and remove the rule-engine change's refusal of the field.  
  Verify both affix scenarios pass in CI, and the rule-engine change's `affix` refusal test is gone.  
  - 2026-11-10 (UTC), the 15 tests of `tests/affix.rs` and `affix_expands_as_vale_does_but_where_decided`, which compares 39 alerts with Vale, pass in the [CI run on PR 46][pr46-ci], at head [144ffd3][c-pr46].  
    `affix_is_refused` is gone from `tests/rules.rs`.  
    Kody left no thread on [PR 46][hakkiri#46].

## 2. Long inputs

- [x] 2.1 Run `spelling`, and `existence` and `substitution` with `affix`, over the rule-engine change's 50,000-character line and 60,000-line file.  
  Verify both tests pass in CI with no panic, and record their run time in the evidence note.  
  - 2026-11-10 (UTC), `spelling_and_affix_over_a_line_of_fifty_thousand_characters` and `spelling_and_affix_over_a_file_of_sixty_thousand_lines` pass in the [CI run on PR 46][pr46-ci].  
    The seven tests of `tests/long_inputs.rs` ran together in a debug build and finished in 99.11 s.  
    They ran in parallel, so each time is measured from the start of the binary: the line test finished at 2.3 s and the file test at 59.0 s.

## 3. Integration

- [x] 3.1 Verify the pull request's CI passes: build, lint, tests, docs, deny, prose lint, OpenSpec and the Template check.  
  - 2026-11-10 (UTC), CI passes on [144ffd3][c-pr46] of [PR 46][hakkiri#46]: Verify, Build and test, Static checks, CodeQL, both Analyze jobs, secrets and the Kody review.  
    The Template check and Publish checks skip on this branch, and I did not run them.

## Workflow follow-up

- Archive this change after the pull request merges, then archive `HKR-020` with `qctl close-from-openspec`.

[hakkiri#41]: https://github.com/victor-software-house/hakkiri/pull/41
[hakkiri#44]: https://github.com/victor-software-house/hakkiri/pull/44
[hakkiri#46]: https://github.com/victor-software-house/hakkiri/pull/46
[pr41-ci]: https://github.com/victor-software-house/hakkiri/actions/runs/42901297915
[pr46-ci]: https://github.com/victor-software-house/hakkiri/actions/runs/42907387841
[c-pr46]: https://github.com/victor-software-house/hakkiri/commit/144ffd32908dfe423992b0703263efc5c00feba4
[case]: ../2026-11-10-case-and-sequence-checks/proposal.md
[spelling]: specs/spelling-check/spec.md
