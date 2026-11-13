## ADDED Requirements

### Requirement: An existence match across a code span is a match

hakkiri SHALL report an `existence` match that runs across a code span, as Vale does, and SHALL drop one that starts or ends on a span.
Vale 3.26.0 masks the span as a run of `*` and reports a match that starts on it as well.
The matched text differs: hakkiri prints `x` for the span, and Vale prints the mask.

#### Scenario: A chain across a code span

- **WHEN** the input is ``The loop reads `a`; the store keeps `b`, which is slow.`` under `ClauseChain`
- **THEN** hakkiri and Vale both give an alert, and the two print the span differently in the matched text ([existence-span-mask 1][existence-span-mask-1])

[existence-span-mask-1]: ../../design.md#1-a-pattern-check-reads-a-code-span-as-one--and-drops-a-match-only-at-its-edges
