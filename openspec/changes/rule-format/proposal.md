# Proposal

Queue row: `HKR-002` in [tasks.yaml][ledger].

## Motivation

The rule engine, the tree and the Vale importer all read the rule format.  
So it must be settled before any of them is built.

The [bootstrap design][bootstrap] fixed the model: prose checks from Vale plus rates, and the project config in `.ctl/hakkiri.yaml`.  
It took structure terms from ast-grep; this change replaces them with Vale's `scope`, which every ProseStyle rule already uses (operator, 2026-11-09).

This change settles the file format, the field names, the scopes and the check types.  
Each is one decision, with every candidate shown side by side.

## Changes

- A spec for rule files: the file format, every field, the scopes, the check types and the test cases.
- A spec for `.ctl/hakkiri.yaml`: which files a project lints, which packages it pins, and the level of each rule.
- Every enabled ProseStyle rule and every frequency-habit rule written in the format, as the proof that it covers the package.
- The intended differences from Vale, such as counting only prose words in a sentence.

## Capabilities

### New Capabilities

- `rule-files`: the format of a rule file and what each field means.
- `project-config`: the format of `.ctl/hakkiri.yaml`.

### Modified Capabilities

None.

## Impact

- No code yet. The tree slice `HKR-003`, the engine slice `HKR-004` and the importer slice `HKR-007` build against these specs.

[ledger]: ../../../tasks.yaml
[bootstrap]: ../archive/2026-11-09-bootstrap-hakkiri/design.md
