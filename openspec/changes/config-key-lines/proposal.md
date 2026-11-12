# Proposal

Queue row: `HKR-053` in [tasks.yaml][ledger].

## Motivation

A fault in a config gave its line from a text search for the key.
The search found a key only at the start of a line, so a config written in flow style gave no line.
It also took the first line after the previous match that read as the key, so two globs that set one rule gave the second glob's line to the first glob's fault.
ctl-core 0.7.1 places a path through the location index, with one step for each key.

## Changes

- The `packages`, `formats`, glob, setting and `vocab` keys of a config take their line from `Input::place`.
- A layer keeps its `Input`, so the check of its keys and the faults read one parsed document.
- hakkiri takes ctl-core 0.7.1.
- The `project-config` spec states that a fault names the line of the key it is about.

## Capabilities

### Modified Capabilities

- `project-config`: a fault names the line of its own key.

## Impact

- Changed: `src/config.rs`, `Cargo.toml`, `Cargo.lock`, `tests/rule_setting.rs`.
- No new dependency.

[ledger]: ../../../tasks.yaml
