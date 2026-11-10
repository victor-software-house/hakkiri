# Design

## Context

The [`vale-differences` design][first-design] sets how the list is built: one scenario for each difference, with a link to its decision, and only what is true now.  
This change adds the part that comes from the [case-and-sequence][case] and [spelling-and-affix][spelling] designs, and from the [rule-engine][engine] and [bootstrap][bootstrap] designs for the regex limit.

## Goals / Non-Goals

**Goals:**

- Every difference in the tagger, the dictionary reader and the spelling check, each as a scenario.
- Every input where Vale panics, stalls or runs out of memory and hakkiri does not.

**Non-Goals:**

- New decisions: each scenario records what its design decided.
- Any measurement of Vale that no design recorded.

## Decisions

### 1. Resource use is listed

Case-and-sequence decisions 3, 5 and 6 and the bootstrap design record cases where Vale runs out of memory, takes cubic or exponential time, or panics, and hakkiri does not.  
Output stays the same, but the behaviour differs, and a project that moves from Vale meets it.  
The last requirement of the spec holds them.

Alternative: list only differences in what a rule reports. Lost: a long line that makes Vale panic is a difference a reader looks for.

### 2. A number is quoted only as its design states it

A scenario gives a time or a size only where the design gives it: 31 ms for 16,667 words, 18 seconds for 8,000 words, and 13 seconds at a 6 GB limit.

Alternative: restate each case as a complexity class alone. Lost: the measured numbers are the evidence that the case matters.

## Risks / Trade-offs

- [A cost is read as a guarantee] → each scenario states the case and the cost the design measured, and no bound beyond it.

[first-design]: ../vale-differences/design.md
[case]: ../archive/2026-11-10-case-and-sequence-checks/design.md
[spelling]: ../archive/2026-11-10-spelling-and-affix-checks/design.md
[engine]: ../archive/2026-11-10-rule-engine/design.md
[bootstrap]: ../archive/2026-11-09-bootstrap-hakkiri/design.md
