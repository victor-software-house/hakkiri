# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
This change starts after the [rule-engine change][engine] merges.

## 1. Open decisions

- [x] 1.1 Record the operator's answer to both open decisions in [design.md][design], moving each into Decisions with the candidates that lost.  
  Change the [readability spec][readability] if the first answer differs from its recommendation.  
  Verify the Open decisions section is empty and `mise run specs:budget` passes.  
  2026-11-10, the operator's answers are recorded as decisions, and the Open decisions section is gone.

## 2. Pattern checks

- [x] 2.1 Build `consistency`: one pattern per `either` pair in Vale's word template, and the forms seen per file in the rule.  
  Give one alert per node, at the last match of the pair.  
  Verify every consistency scenario in the [pattern-checks spec][pattern] passes as a unit test in CI.
  - 2026-11-10 (UTC), 13 tests in `tests/consistency.rs` cover every consistency scenario, in a Linux box run of [94c5649][c-consistency].  
    They add a mixed paragraph, the last match, the other form first, case, `nonword`, code, a fresh file, and two refusals.
- [x] 2.2 Build `conditional`: group detection, the plain presence check, the terms captured per file in document order, `exceptions`, and the refusal of `in`.  
  Verify every conditional scenario passes as a unit test in CI.
  - 2026-11-10 (UTC), 15 tests in `tests/conditional.rs` cover every conditional scenario, in a Linux box run of [b4ae3a2][c-conditional].  
    They add a definition in the same block, case, an exception, code, groups that are not groups, and two refusals.
- [x] 2.3 Build `repetition`: the joined pattern, the trimmed comparison, `alpha`, `max` with Vale's count, the one-sentence check and `exceptions`.  
  Verify every repetition scenario passes as a unit test in CI.
  - 2026-11-10 (UTC), 11 tests in `tests/repetition.rs` cover every repetition scenario, in a Linux box run of [ec04711][c-repetition].  
    They add the count's reset, a repeat over two sentences that still counts, case, `alpha`, a line wrap, and code.

## 3. Readability

- [x] 3.1 Build `readability` on the engine's scores: the average, the strict comparison and the default scope at line 1.  
  Add the per-node scope and the refusal the open decision chose.  
  Verify every [readability scenario][readability] passes as a unit test in CI.
  - 2026-11-10 (UTC), 14 tests in `tests/readability_check.rs` cover every readability scenario, in a Linux box run of [cfb9839][c-readability].  
    They add a repeated score, a missing list, a grade that is not a number, and a list item.

## 4. Long inputs

- [ ] 4.1 Run the four check types over the rule-engine change's 50,000-character line and 60,000-line file.  
  Verify both tests pass in CI with no panic, and record their run time in the evidence note.

## 5. Integration

- [ ] 5.1 Verify the pull request's CI passes: build, lint, tests, docs, deny, prose lint, OpenSpec and the Template check.

## Workflow follow-up

- Archive this change after the pull request merges, then archive `HKR-013` with `qctl close-from-openspec`.

[engine]: ../archive/2026-11-10-rule-engine/proposal.md
[design]: design.md
[pattern]: specs/pattern-checks/spec.md
[readability]: specs/readability-check/spec.md
[c-readability]: https://github.com/victor-software-house/hakkiri/commit/cfb983962d9399c5d3c0865afc81eb93390c82cd
[c-repetition]: https://github.com/victor-software-house/hakkiri/commit/ec047118bdfc7d89163072067848beaf6704a874
[c-consistency]: https://github.com/victor-software-house/hakkiri/commit/94c564949d03f8da42fdf3a2e50eee6216954a0e
[c-conditional]: https://github.com/victor-software-house/hakkiri/commit/b4ae3a23b012a68da4157af139f6aefa63d19e4a
