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

- [ ] 3.1 Verify the pull request's CI passes: build, lint, tests, prose lint, OpenSpec and the Template check.
- [ ] 3.2 Verify the operator approved the format on the pull request.

## Workflow follow-up

- Archive this change after the pull request merges, then archive `HKR-002` with `qctl close-from-openspec`.

[design]: design.md
[rules]: prose-style-rules.md
