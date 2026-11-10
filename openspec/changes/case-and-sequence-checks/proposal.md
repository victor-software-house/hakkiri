# Proposal

Queue row: `HKR-014` in [tasks.yaml][ledger], a new row after `HKR-013`.

## Motivation

The [`rule-engine` change][engine] and the [`pattern-and-readability-checks` change][pattern] run eight of Vale's eleven check types.  
The Vale importer, `HKR-007`, also meets `capitalization` and `sequence` in the upstream styles it converts.  
These two need data the engine does not hold: part-of-speech tags and title case word lists.  
The `spelling` and `affix` checks, which need a dictionary with its affix rules, are in the [`spelling-and-affix-checks` change][spelling].

ProseStyle uses none of them, so no project that hakkiri lints today waits on this change.

## Changes

- `capitalization`: `$title` in AP or Chicago style, `$sentence`, `$lower`, `$upper` or a pattern, with Vale's word share `threshold`.
- `sequence`: tokens matched word by word in a sentence, by pattern, part-of-speech tag or universal tag, with `skip`, `negate`, `min` and `target`.

## Capabilities

### New Capabilities

- `capitalization-check`: the `capitalization` check.
- `sequence-check`: the `sequence` check.

### Modified Capabilities

None.

## Impact

- Ported code and data for the tagger, the dictionary and its reader, as the [design][design] decides.
- One new module per check type, with unit tests.

[ledger]: ../../../tasks.yaml
[engine]: ../archive/2026-11-10-rule-engine/proposal.md
[pattern]: ../archive/2026-11-10-pattern-and-readability-checks/proposal.md
[design]: design.md
[spelling]: ../spelling-and-affix-checks/proposal.md
