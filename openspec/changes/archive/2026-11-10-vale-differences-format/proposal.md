# Proposal

Queue row: `HKR-025` in [tasks.yaml][ledger].  
It is the last of four changes that build the `vale-differences` spec, after [`vale-differences`][first], [`vale-differences-checks`][checks] and [`vale-differences-readers`][readers].

## Motivation

The first three changes list the differences that the designs record as differences.  
The list has any behaviour that differs from Vale, whether or not a design calls it a difference.

Checked against Vale 66bbb85, three decisions that the designs treat as choices of format differ in behaviour:

- Comments read as Markdown.
- The `table.row` scope.
- The rule fields, check types and configuration that hakkiri defines for itself.

## Changes

- The `vale-differences` spec gains three requirements:
  - Comments read as Markdown.
  - What a rule file accepts, refuses or selects where Vale does otherwise.
  - The configuration and packages.
- Each difference is a scenario with a link to the decision that records it.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `vale-differences`: three requirements added.

## Impact

- No code changes.
- This change adds to the spec that [`vale-differences`][first] creates, so it is archived after that change and the two before it.

[ledger]: ../../../../tasks.yaml
[first]: ../2026-11-10-vale-differences/proposal.md
[checks]: ../2026-11-10-vale-differences-checks/proposal.md
[readers]: ../2026-11-10-vale-differences-readers/proposal.md
