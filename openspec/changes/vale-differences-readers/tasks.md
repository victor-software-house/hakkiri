# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
This change adds to the spec that [`vale-differences`][first] creates.

## 1. The list

- [ ] 1.1 Add three requirements: the ported readers, the inputs on which a run finishes in hakkiri and fails or stalls in Vale, and the known gaps.  
  Each scenario links its decision or row.  
  Verify `openspec validate --all --strict` and the link check pass in CI, and that each scenario's link resolves to its decision.

## 2. Integration

- [ ] 2.1 Verify the pull request's CI passes: lint, prose lint, OpenSpec and the Template check.

## Workflow follow-up

- Archive this change after [`vale-differences-checks`][checks] is archived, then archive `HKR-024` with `qctl close-from-openspec`.

[first]: ../vale-differences/proposal.md
[checks]: ../vale-differences-checks/proposal.md
