# Design

## Context

Each change that built a check recorded the places where hakkiri departs from Vale 3.26.0 in its own design.  
The [format design][format-7] holds the first list, and the [rule-engine][engine], [pattern-and-readability][pattern], [case-and-sequence][case] and [spelling-and-affix][spelling] designs add to it.  
The [bootstrap design][bootstrap] and the [sentence-splitter design][splitter] hold two more.

## Goals / Non-Goals

**Goals:**

- One current list of every intended difference, each with a link to its decision.
- A rule that keeps the list current.

**Non-Goals:**

- New decisions: the spec records what the designs decided and changes none of it.
- Mixing gaps into the differences: a gap, where hakkiri lacks something Vale does and means to close it, closes, and a difference stays. Gaps are listed apart, under Known gaps, by [`vale-differences-readers`][readers].
- Differences that were never intended: a defect in hakkiri is fixed, not listed.

## Decisions

### 1. One spec, one scenario for each difference, in four changes

The spec is `openspec/specs/vale-differences/spec.md`.  
A requirement groups the scenarios that share a subject, and each scenario states the case, Vale's behaviour and hakkiri's, with a link to its decision.  
The whole list is over 3,000 words, and `specs:budget` allows 1,200 for one change's spec, so [`vale-differences-checks`][checks], [`vale-differences-readers`][readers] and [`vale-differences-format`][format] add their requirements to this spec once this change has created it.

Alternatives:

- A table in the spec body. Lost: OpenSpec checks only requirements and scenarios, so a table could drift with no check, and it counts toward the 500-character limit of a requirement.
- A page outside `openspec/specs`. Lost: every other behaviour of hakkiri is a spec there, and a change updates specs.
- Leave the differences in the designs. Lost: archived designs are not edited, so no current list exists.
- One spec for each area. Lost: the outcome is one list, and a reader would open four.

### 2. The list holds what is true now

Decision 7 of the format design lists a `~` row that [case-and-sequence decision 9][case-9] replaced: `~list` and `~blockquote` now keep what Vale keeps.  
The spec lists the two cases that decision 9 leaves as differences, `sentence & paragraph` and `~paragraph`, and not the row it replaced.  
The scenario that closes the first requirement states this rule.

Alternative: copy the format list as written. Lost: it would list a difference that no longer exists.

### 3. The rule lives in the spec, `openspec/config.yaml` and `AGENTS.md`

The first requirement states it as a behaviour of the repository.  
`openspec/config.yaml` puts it in the rules for specs, which OpenSpec prints for every change that writes a spec.  
`AGENTS.md` puts it in the contract that a human or agent reads before any change.

Alternative: the spec alone. Lost: an author who writes a delta for another capability would not open this spec.

## Risks / Trade-offs

- [A design's wording is read as a difference it did not intend] → each scenario quotes the design's own cases, and a reviewer checks each against its decision.
- [A decision adds a difference and no one updates the list] → the rule is in three places, and a reviewer checks it.

[checks]: ../vale-differences-checks/proposal.md
[readers]: ../vale-differences-readers/proposal.md
[format-7]: ../archive/2026-11-10-rule-format/design.md#7-intended-differences-from-vale
[engine]: ../archive/2026-11-10-rule-engine/design.md
[pattern]: ../archive/2026-11-10-pattern-and-readability-checks/design.md
[case]: ../archive/2026-11-10-case-and-sequence-checks/design.md
[spelling]: ../archive/2026-11-10-spelling-and-affix-checks/design.md
[bootstrap]: ../archive/2026-11-09-bootstrap-hakkiri/design.md
[splitter]: ../archive/2026-11-10-sentence-splitter/design.md
[case-9]: ../archive/2026-11-10-case-and-sequence-checks/design.md#9-a-block-of-text-has-the-scope-of-one-element
[format]: ../vale-differences-format/proposal.md
