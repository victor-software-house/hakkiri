# Proposal

Queue row: `HKR-024` in [tasks.yaml][ledger].  
It is the third of four changes that build the `vale-differences` spec, after [`vale-differences`][first] and [`vale-differences-checks`][checks], and before [`vale-differences-format`][format].

## Motivation

The first two changes list what a rule reads, where an alert points, how text is measured, the pattern checks and the refusals.  
The rest of the list is where hakkiri's ports of Vale's tagger, dictionary reader and spelling check differ in detail, and where a run that would fail or stall in Vale finishes.  
A gap, where hakkiri lacks something Vale does and means to close it, is not a difference, so the spec lists each apart, with its queue row.

## Changes

- The `vale-differences` spec gains three requirements: the differences in the ported readers, the inputs on which a run finishes in hakkiri and panics, stalls or runs out of memory in Vale, and the known gaps.
- A language other than English joins the differences, linked to the spelling change's non-goal.
- The known gaps are the text inside HTML blocks, `Vale.Spelling`, vocabularies and the Markdown parser, each with its queue row.
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
