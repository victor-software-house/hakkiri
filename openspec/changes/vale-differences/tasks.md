# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
This change reads the designs that [`spelling-and-affix-checks`][spelling] and the changes before it archived.

## 1. The list

- [ ] 1.1 Write the first four requirements of the `vale-differences` spec.  
  They state that every difference is listed, and give the differences in what a rule reads, where an alert points and how text is measured.  
  Each scenario links its decision.  
  Verify `openspec validate --all --strict` and the link check pass in CI, and that each scenario's link resolves to its decision.

## 2. The rule

- [ ] 2.1 Add the rule that a change adding, changing or removing a difference updates the spec in the same pull request, to `openspec/config.yaml` and `AGENTS.md`.  
  Verify `openspec instructions specs --change vale-differences` prints the rule, and `AGENTS.md` holds it.

## 3. Integration

- [ ] 3.1 Verify the pull request's CI passes: lint, prose lint, OpenSpec and the Template check.

## Workflow follow-up

- Archive this change first after the pull request merges, then [`vale-differences-checks`][checks], [`vale-differences-readers`][readers] and [`vale-differences-format`][format], each in turn, and archive `HKR-022` to `HKR-025` with `qctl close-from-openspec`.

[spelling]: ../archive/2026-11-10-spelling-and-affix-checks/proposal.md
[checks]: ../vale-differences-checks/proposal.md
[readers]: ../vale-differences-readers/proposal.md
[format]: ../vale-differences-format/proposal.md
