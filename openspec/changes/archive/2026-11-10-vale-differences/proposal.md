# Proposal

Queue row: `HKR-022` in [tasks.yaml][ledger].  
The list is long enough to exceed the spec budget, so it comes in four changes that add to one spec: this one, [`vale-differences-checks`][checks] (`HKR-023`), [`vale-differences-readers`][readers] (`HKR-024`) and [`vale-differences-format`][format] (`HKR-025`).

## Motivation

hakkiri differs from Vale 3.26.0 on purpose in many places.  
The differences sit in the decisions of six archived changes, with the [format design][format-7] as the first list.  
No page names them all, and a later decision has already changed one of them: [case-and-sequence decision 9][case-9] replaced the `~` row of the format list.

A project that moves from Vale to hakkiri needs one current list to read, and each change that adds a difference needs one place to update.

## Changes

- A new spec, `vale-differences`, with a Purpose and four requirements: the rule that every difference is listed, and the differences in what a rule reads, where an alert points and how text is measured.
- Each difference is a scenario: the case, what Vale does, what hakkiri does, and a link to the decision that records it.
- The rule that a change adding, changing or removing a difference updates the spec in the same pull request goes into the spec, `openspec/config.yaml` and `AGENTS.md`.

## Capabilities

### New Capabilities

- `vale-differences`: the list of intended differences from Vale.

### Modified Capabilities

None.

## Impact

- No code changes.
- The spec joins `openspec/specs/` when this change is archived, and the three later changes then add their requirements to it.

[ledger]: ../../../../tasks.yaml
[checks]: ../2026-11-10-vale-differences-checks/proposal.md
[readers]: ../2026-11-10-vale-differences-readers/proposal.md
[format-7]: ../2026-11-10-rule-format/design.md#7-intended-differences-from-vale
[case-9]: ../2026-11-10-case-and-sequence-checks/design.md#9-a-block-of-text-has-the-scope-of-one-element
[format]: ../2026-11-10-vale-differences-format/proposal.md
