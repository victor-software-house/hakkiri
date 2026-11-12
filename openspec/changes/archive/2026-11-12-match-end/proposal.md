# Proposal

Queue row: `HKR-047` in [tasks.yaml][ledger].

## Motivation

An alert for a match that wraps a line ended at the end of its first line, and no field named the line the match ends on.
A tool that reads the JSON could not mark the match, and the end column of such an alert pointed at nothing in the match.
The operator ruled on 2026-11-11 that an alert names the true end of the match.

## Changes

- The JSON alert gains `end_line`, and `end_line` and `end_column` name the last character of the match, also for a match on one line.
- The pretty line keeps `path:line:column`.
- The alerts spec states the end, and the instructions describe the fields.
- The `vale-differences` spec records that Vale prints the first column plus the length of the match, which can pass the end of the line.
- The comparison with Vale in `tests/vocabulary_rules.rs` checks the end of a wrapped match through that count.

## Capabilities

### Modified Capabilities

- `alerts`: a match ends at its last character, on its own line.
- `check-command`: each JSON alert holds `end_line`.
- `vale-differences`: where a wrapped match ends.

## Impact

- Changed: `src/alert.rs`, `src/report.rs`, `src/instructions.md`, `tests/alerts.rs`, `tests/check.rs` and `tests/vocabulary_rules.rs`.
- The `Alert` struct gains `last_line`.
- No new dependency.

[ledger]: ../../../../tasks.yaml
