# Proposal

Queue row: `HKR-050` in [tasks.yaml][ledger].
It follows [`plain-text-files`][plain], which made a file with no reader plain text for `hakkiri check`, and changes [test-command][test].

## Motivation

A test case names its `format`, an extension, and hakkiri reads the input as a file of that extension.
`hakkiri test` refused a format that no language reads, such as `txt`, though `hakkiri check` now reads such a file as plain text.
The ProseStyle package could not test that a text file is not prose.
A `formats` entry that maps an extension to a reader changed nothing for a case.

## Changes

- A case with a format that no extension names reads its input as plain text.
- A case that runs through the config reads its input by the reader a `formats` entry gives `input.<format>`, before the extension, as `check` does.
- An isolated case reads by the extension, or as plain text.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `test-command`: the reader of a case.

## Impact

- Changed: `src/suite.rs`, `src/instructions.md`, the skill and the `test-command` spec.
- No new dependency.

[ledger]: ../../../tasks.yaml
[plain]: ../plain-text-files/proposal.md
[test]: ../../specs/test-command/spec.md
