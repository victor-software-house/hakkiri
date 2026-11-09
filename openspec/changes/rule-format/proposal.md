# Proposal

Queue row: `HKR-002` in [tasks.yaml][ledger].

## Motivation

The rule engine, the tree and the Vale importer all read the rule format.  
So it must be settled before any of them is built.

The [bootstrap design][bootstrap] fixed the model.  
Structure terms come from ast-grep, and prose checks from Vale plus rates.  
The project config lives in `.ctl/hakkiri.yaml`.  
This change settles the file format, the field names, the node kinds and the check types.  
Each is one decision, with every candidate shown side by side.

## Changes

- A spec for rule files: the file format, every field, the node kinds, the check types and the test cases.
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
