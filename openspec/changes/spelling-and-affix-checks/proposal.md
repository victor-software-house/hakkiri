# Proposal

Queue row: `HKR-020` in [tasks.yaml][ledger], a new row after `HKR-014`.

## Motivation

The [`case-and-sequence-checks` change][case] runs `capitalization` and `sequence`, and builds the dictionary reader and the built-in dictionary.  
The Vale importer, `HKR-007`, also meets `spelling` in the upstream styles it converts, and Vale's `affix` field on `existence` and `substitution`.  
Both need that dictionary and its affix rules.

ProseStyle uses neither, so no project that hakkiri lints today waits on this change.

## Changes

- `spelling`: words checked against a dictionary, Vale's built-in English one by default, with `filters`, `ignore` word lists, `split` and custom dictionaries.
- `affix`: an `existence` token or a `substitution` key written `word/FLAGS` matches every form its flags name.

## Capabilities

### New Capabilities

- `spelling-check`: the `spelling` check, dictionaries, and `affix`.

### Modified Capabilities

None.

## Impact

- The rule-engine change's refusal of `affix` ends here.
- One new module for the `spelling` check, with unit tests.

[ledger]: ../../../tasks.yaml
[case]: ../case-and-sequence-checks/proposal.md
