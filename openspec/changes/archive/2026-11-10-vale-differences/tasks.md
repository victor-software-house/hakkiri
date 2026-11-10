# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
This change reads the designs that [`spelling-and-affix-checks`][spelling] and the changes before it archived.

## 1. The list

- [x] 1.1 Write the first four requirements of the `vale-differences` spec.  
  They state that every difference is listed, and give the differences in what a rule reads, where an alert points and how text is measured.  
  Each scenario links its decision.  
  Verify `openspec validate --all --strict` and the link check pass in CI, and that each scenario's link resolves to its decision.  
  - 2026-11-10 (UTC), `openspec validate --all --strict` and the markdown link check pass in the [CI run on PR 48][pr48-ci], at head [9a09b1c][c-pr48].  
    A dry-run archive of the four changes gave one spec of 12 requirements and 62 scenarios that validates strictly, and every link in it resolves to an archived design.

## 2. The rule

- [x] 2.1 Add the rule that a change adding, changing or removing a difference updates the spec in the same pull request, to `openspec/config.yaml` and `AGENTS.md`.  
  Verify `openspec instructions specs --change vale-differences` prints the rule, and `AGENTS.md` holds it.  
  - 2026-11-10 (UTC), on `main` after [PR 48][hakkiri#48] merged, `openspec instructions specs --change vale-differences` printed the rule, and `AGENTS.md` holds it under Rules.

## 3. Integration

- [x] 3.1 Verify the pull request's CI passes: lint, prose lint, OpenSpec and the Template check.  
  - 2026-11-10 (UTC), CI passes on [9a09b1c][c-pr48] of [PR 48][hakkiri#48]: Verify, Build and test, Static checks, CodeQL, both Analyze jobs, secrets and the Kody review.  
    The Template check and Publish checks skip on this branch, and I did not run them.  
    Kody left four threads on PR 48, all fixed in that PR before the merge.

## Workflow follow-up

- Archive this change first after the pull request merges, then [`vale-differences-checks`][checks], [`vale-differences-readers`][readers] and [`vale-differences-format`][format], each in turn, and archive `HKR-022` to `HKR-025` with `qctl close-from-openspec`.

[spelling]: ../2026-11-10-spelling-and-affix-checks/proposal.md
[checks]: ../2026-11-10-vale-differences-checks/proposal.md
[readers]: ../2026-11-10-vale-differences-readers/proposal.md
[format]: ../2026-11-10-vale-differences-format/proposal.md
[hakkiri#48]: https://github.com/victor-software-house/hakkiri/pull/48
[pr48-ci]: https://github.com/victor-software-house/hakkiri/actions/runs/42915518615
[c-pr48]: https://github.com/victor-software-house/hakkiri/commit/9a09b1c1ba094d55999197d50d735dff0e88f781
