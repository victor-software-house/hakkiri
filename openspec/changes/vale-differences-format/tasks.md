# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
This change adds to the spec that [`vale-differences`][first] creates.

## 1. The list

- [ ] 1.1 Add the three requirements for comments, for what a rule file accepts or refuses, and for the configuration and packages, each scenario with a link to its decision.  
  Verify `openspec validate --all --strict` and the link check pass in CI, and that each scenario's link resolves to its decision.

## 2. Integration

- [ ] 2.1 Verify the pull request's CI passes: lint, prose lint, OpenSpec and the Template check.

## Workflow follow-up

- Archive this change after [`vale-differences-readers`][readers] is archived, then archive `HKR-025` with `qctl close-from-openspec`.

[first]: ../vale-differences/proposal.md
[readers]: ../vale-differences-readers/proposal.md
