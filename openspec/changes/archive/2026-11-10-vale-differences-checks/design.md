# Design

## Context

The [`vale-differences` design][first-design] sets how the list is built: one scenario for each difference, with a link to its decision, and only what is true now.  
This change adds the part that comes from the [`pattern-and-readability`][pattern], [case-and-sequence][case] and [spelling-and-affix][spelling] designs.

## Goals / Non-Goals

**Goals:**

- Every difference in how `consistency`, `conditional` and `repetition` run, each as a scenario.
- Every rule that hakkiri refuses at load where Vale loads it, each as a scenario.

**Non-Goals:**

- New decisions: each scenario records what its design decided.

## Decisions

### 1. The refusals form one requirement

Eleven rules that Vale loads and hakkiri refuses sit in one requirement, from three designs.  
A project that moves a Vale style needs to know which of its rules stop loading, and they are the same kind of difference.

Alternative: put each refusal beside the check it belongs to. Lost: a reader would scan every check to find what stops loading.

## Risks / Trade-offs

- [A refusal is read as covering a case its decision left open] → each scenario names the field and quotes the design's case.

[first-design]: ../2026-11-10-vale-differences/design.md
[pattern]: ../2026-11-10-pattern-and-readability-checks/design.md
[case]: ../2026-11-10-case-and-sequence-checks/design.md
[spelling]: ../2026-11-10-spelling-and-affix-checks/design.md
