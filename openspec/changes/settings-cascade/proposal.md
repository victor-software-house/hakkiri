# Proposal

Queue row: `HKR-069` in [tasks.yaml][ledger].

## Motivation

hakkiri applies the globs that match a file in the order written, packages first, and a later glob wins for the same key, as Vale applies its sections ([file.go][vale-file]).  
So a project glob loses to any glob written after it, however much more specific it is.  
The operator ruled on 2026-11-13 that settings resolve as CSS rules do: every setting that matches a file merges, only settings of the same key compete, and for one key the most specific glob wins.

## Changes

- Each glob has a specificity, read from its text, and the globs that match a file apply from the least specific to the most, so the more specific wins for the same key.
- Turning a rule on or off and its level resolve as two keys, so a more specific `off` keeps a level set by a less specific glob, as a later `off` keeps it today.
- An empty `styles` list drops the rule settings of the globs that rank below it.
- A glob with `{…}` groups ranks, for each file, as the most specific of its alternatives that matches that file (row `HKR-072`).
- The `project-config` and `vale-differences` specs state the rank, and the open `code-span-size` and `rule-parameters` changes drop their written-order wording.
- How a tie between a package and the project breaks, and how the project forces a value with `!override`, is the change [package-cascade][package-cascade] (row `HKR-071`), which builds on this one.

## Capabilities

### Modified Capabilities

- `project-config`: the order in which the settings of the matching globs apply.
- `vale-differences`: hakkiri ranks the matching globs by specificity, where Vale applies its sections in the order written.

## Impact

- Changed: `src/config.rs`, the config tests, `src/instructions.md`, the skill and `AGENTS.md`.
- A config whose globs overlap can resolve differently: a more specific glob now wins wherever it is written.  
  The scenarios that showed written order are rewritten to show the rank.

[ledger]: ../../../tasks.yaml
[package-cascade]: ../package-cascade/proposal.md
[vale-file]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/file.go#L185-L215
