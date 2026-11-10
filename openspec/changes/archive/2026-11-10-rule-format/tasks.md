# Tasks

## 1. Decisions

- [x] 1.1 Show the operator each format decision with every candidate side by side, and verify each answer is recorded in [design.md][design] with its alternatives.
  - 2026-11-10 (UTC), decisions 1 to 9 are recorded, each with its alternatives.
- [x] 1.2 Measure each claimed Vale behaviour with Vale 3.26.0, or read it in Vale's source, and verify [design.md][design] names the measurement.
  - 2026-11-10 (UTC), a word-count rule measured code spans, links, bare URLs and autolinks.  
    A scope rule measured `~blockquote` on a list inside a quote.  
    The Rust comment query and the scope operators were read at Vale commit `66bbb85`.

## 2. Specs

- [x] 2.1 Write the `rule-files` and `project-config` specs, and verify `openspec validate rule-format --strict` passes.
  - 2026-11-10 (UTC), `openspec validate rule-format --strict` reported the change valid.
- [x] 2.2 Write every ProseStyle rule and habit candidate in the format, and verify each sample parses with no lookbehind for structure.
  - 2026-11-10 (UTC), [prose-style-rules.md][rules] holds all 25 rules and 8 habit rules; each of its 8 YAML samples parsed with `yq`.

## 3. Integration

- [x] 3.1 Verify the pull request's CI passes: build, lint, tests, prose lint, OpenSpec and the Template check.
  - 2026-11-10 (UTC), [Verify][run-pr3] passed on [PR 3][pr3] at `e63c150`, and [Verify][run-pr4] passed on [PR 4][pr4] at `210b1f6`.  
    The Template check skipped: neither pull request changed template files.
- [x] 3.2 Verify the operator approved the format on the pull request.
  - 2026-11-10 (UTC), the operator chose each format decision (operator, 2026-11-09 and 2026-11-10).  
    The operator then directed the merge once every review thread was answered and resolved (operator, 2026-11-10).  
    Kody's findings on [PR 3][pr3] and [PR 4][pr4] were fixed in [PR 4][pr4] and resolved, and the stack merged at `4646cb0`.

## Workflow follow-up

- Archive this change after the pull request merges, then archive `HKR-002` with `qctl close-from-openspec`.

[design]: design.md
[rules]: prose-style-rules.md
[pr3]: https://github.com/victor-software-house/hakkiri/pull/3
[pr4]: https://github.com/victor-software-house/hakkiri/pull/4
[run-pr3]: https://github.com/victor-software-house/hakkiri/actions/runs/42769690302
[run-pr4]: https://github.com/victor-software-house/hakkiri/actions/runs/42770664229
