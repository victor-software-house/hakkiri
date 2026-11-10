# Proposal

Queue row: `HKR-023` in [tasks.yaml][ledger].  
It is the second of four changes that build the `vale-differences` spec, after [`vale-differences`][first] and before [`vale-differences-readers`][readers].

## Motivation

The [`vale-differences` change][first] lists the differences in what a rule reads, where an alert points and how text is measured.  
The pattern checks, and the rules that hakkiri refuses where Vale loads them, are the next part of the list.  
A project that moves a Vale style meets a refusal at load, so it needs to find each one in one place.

## Changes

- The `vale-differences` spec gains two requirements: the differences in how the pattern checks run, and the rules that hakkiri refuses at load where Vale loads them.
- Each difference is a scenario with a link to the decision that records it.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `vale-differences`: two requirements added.

## Impact

- No code changes.
- This change adds to the spec that [`vale-differences`][first] creates, so it is archived after that change.

[ledger]: ../../../tasks.yaml
[first]: ../vale-differences/proposal.md
[readers]: ../vale-differences-readers/proposal.md
