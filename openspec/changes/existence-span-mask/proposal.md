# Proposal

Queue row: `HKR-060` in [tasks.yaml][ledger].
It splits out of `code-span-tokens`, whose design went over the word budget.

## Motivation

`run_existence` searched the text where a code span is the letter `x`, and dropped each match that overlapped a span.
Vale masks a span as a run of `*`, and a match that runs across one is a match.
Of the 487 alerts of `ClauseChain` that Vale gave and hakkiri did not, 452 held a span inside the match.

## Changes

- An `existence` rule reads a code span as one `*`, as Vale masks it, and drops a match only when it starts or ends on a span, so a match across a span is a match.
- The `scope-selection` and `vale-differences` specs state it.

## Capabilities

### Modified Capabilities

- `scope-selection`: an `existence` match across a code span is a match.
- `vale-differences`: Vale reports an `existence` match across a code span, and the two print the span differently.

## Impact

- Changed: `src/check.rs`, `src/text.rs`, `tests/alerts.rs`, `tests/fixtures/html-block/departures.tsv`, the skill and `AGENTS.md`.
- On nine repositories `ClauseChain` gives 1,536 alerts where it gave 1,020, and Vale gives 1,492.
  The design lists the counts and the false alerts that remain.

[ledger]: ../../../tasks.yaml
