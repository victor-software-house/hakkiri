# Proposal

Queue row: `HKR-051` in [tasks.yaml][ledger].

## Motivation

The ProseStyle port found two statements about a code span that look opposed.
The `vale-differences` spec holds that a code span counts one word in hakkiri and none in Vale.
ProseStyle `LongSentence` counts the words of a sentence with an `occurrence` rule, and under hakkiri it counts none for a code span.
A probe of both tools on one sentence shows both statements hold, for two different checks.

## Changes

- A test pins that a `metric` rule counts a code span as one word.
- A test pins that an `occurrence` rule counts no match that overlaps a code span, with a token that also reads Vale's mask.
- The `vale-differences` spec records the second case, with Vale's counts.
- The ProseStyle rule moves to a form hakkiri counts, in the prose-style package, as the last task describes.

## Capabilities

### Modified Capabilities

- `vale-differences`: an `occurrence` count of a code span.

## Impact

- Changed: `tests/metric.rs`, `tests/alerts.rs`.
- No change to the engine and no new dependency.

[ledger]: ../../../tasks.yaml
