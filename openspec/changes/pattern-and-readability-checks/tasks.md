# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
This change starts after the [rule-engine change][engine] merges.

## 1. Open decisions

- [x] 1.1 Record the operator's answer to both open decisions in [design.md][design], moving each into Decisions with the candidates that lost.  
  Change the [readability spec][readability] if the first answer differs from its recommendation.  
  Verify the Open decisions section is empty and `mise run specs:budget` passes.  
  2026-11-10, the operator's answers are recorded as decisions, and the Open decisions section is gone.

## 2. Pattern checks

- [ ] 2.1 Build `consistency`: one pattern per `either` pair in Vale's word template, and the forms seen per file in the rule.  
  Give one alert per node, at the last match of the pair.  
  Verify every consistency scenario in the [pattern-checks spec][pattern] passes as a unit test in CI.
- [ ] 2.2 Build `conditional`: group detection, the plain presence check, the terms captured per file in document order, `exceptions`, and the refusal of `in`.  
  Verify every conditional scenario passes as a unit test in CI.
- [ ] 2.3 Build `repetition`: the joined pattern, the trimmed comparison, `alpha`, `max` with Vale's count, the one-sentence check and `exceptions`.  
  Verify every repetition scenario passes as a unit test in CI.

## 3. Readability

- [ ] 3.1 Build `readability` on the engine's scores: the average, the strict comparison and the default scope at line 1.  
  Add the per-node scope and the refusal the open decision chose.  
  Verify every [readability scenario][readability] passes as a unit test in CI.

## 4. Long inputs

- [ ] 4.1 Run the four check types over the rule-engine change's 50,000-character line and 60,000-line file.  
  Verify both tests pass in CI with no panic, and record their run time in the evidence note.

## 5. Integration

- [ ] 5.1 Verify the pull request's CI passes: build, lint, tests, docs, deny, prose lint, OpenSpec and the Template check.

## Workflow follow-up

- Archive this change after the pull request merges, then archive `HKR-013` with `qctl close-from-openspec`.

[engine]: ../rule-engine/proposal.md
[design]: design.md
[pattern]: specs/pattern-checks/spec.md
[readability]: specs/readability-check/spec.md
