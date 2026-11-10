# Proposal

Queue row: `HKR-024` in [tasks.yaml][ledger].  
It is the third of four changes that build the `vale-differences` spec, after [`vale-differences`][first] and [`vale-differences-checks`][checks], and before [`vale-differences-format`][format].

## Motivation

The first two changes list what a rule reads, where an alert points, how text is measured, the pattern checks and the refusals.  
The rest of the list is where hakkiri's ports of Vale's tagger, dictionary reader and spelling check differ in detail, and where a run that would fail or stall in Vale finishes.

## Changes

- The `vale-differences` spec gains two requirements: the differences in the ported readers, and the inputs on which a run finishes in hakkiri and panics, stalls or runs out of memory in Vale.
- Each difference is a scenario with a link to the decision that records it.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `vale-differences`: two requirements added.

## Impact

- No code changes.
- This change adds to the spec that [`vale-differences`][first] creates, so it is archived after that change and [`vale-differences-checks`][checks].

[ledger]: ../../../tasks.yaml
[first]: ../vale-differences/proposal.md
[checks]: ../vale-differences-checks/proposal.md
[format]: ../vale-differences-format/proposal.md
