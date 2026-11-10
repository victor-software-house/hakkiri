# Proposal

Queue row: `HKR-013` in [tasks.yaml][ledger], a new row after `HKR-004`.

## Motivation

The [`rule-engine` change][engine] runs the four check types ProseStyle uses.  
The Vale importer, `HKR-007`, converts every check type but `script`, and the upstream styles it converts use more of them.  
Four of the remaining types need no data beyond what the engine already holds: `consistency`, `conditional`, `repetition` and `readability`.

## Changes

- `consistency`: an alert when a file uses both forms of an `either` pair.
- `conditional`: an alert when `first` matches and `second` does not, with terms that `second` captures counting for the rest of the file.
- `repetition`: an alert when a token match repeats the one before it in the same sentence.
- `readability`: an alert when the average of the named grade scores is over `grade`.

## Capabilities

### New Capabilities

- `pattern-checks`: the `consistency`, `conditional` and `repetition` checks.
- `readability-check`: the `readability` check.

### Modified Capabilities

None.

## Impact

- No new dependency: the scores, sentences and patterns come from the rule-engine change.
- One new module per check type, with unit tests.

[ledger]: ../../../../tasks.yaml
[engine]: ../2026-11-10-rule-engine/proposal.md
