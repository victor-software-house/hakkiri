# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
This change adds to the spec that [`vale-differences`][first] creates.

## 1. The list

- [x] 1.1 Add the three requirements for comments, for what a rule file accepts, refuses or selects, and for the configuration and packages, each scenario with a link to its decision.  
  Verify `openspec validate --all --strict` and the link check pass in CI, and that each scenario's link resolves to its decision.  
  - 2026-11-10 (UTC), `openspec validate --all --strict` and the markdown link check pass in the [CI run on PR 48][pr48-ci], at head [9a09b1c][c-pr48].  
    A dry-run archive of the four changes gave one spec of 12 requirements and 62 scenarios that validates strictly, and every link in it resolves to an archived design.

## 2. Integration

- [x] 2.1 Verify the pull request's CI passes: lint, prose lint, OpenSpec and the Template check.  
  - 2026-11-10 (UTC), CI passes on [9a09b1c][c-pr48] of [PR 48][hakkiri#48]: Verify, Build and test, Static checks, CodeQL, both Analyze jobs, secrets and the Kody review.  
    The Template check and Publish checks skip on this branch, and I did not run them.  
    Kody left four threads on PR 48, all fixed in that PR before the merge.

## Workflow follow-up

- Archive this change after [`vale-differences-readers`][readers] is archived, then archive `HKR-025` with `qctl close-from-openspec`.

[first]: ../2026-11-10-vale-differences/proposal.md
[readers]: ../2026-11-10-vale-differences-readers/proposal.md
[hakkiri#48]: https://github.com/victor-software-house/hakkiri/pull/48
[pr48-ci]: https://github.com/victor-software-house/hakkiri/actions/runs/42915518615
[c-pr48]: https://github.com/victor-software-house/hakkiri/commit/9a09b1c1ba094d55999197d50d735dff0e88f781
