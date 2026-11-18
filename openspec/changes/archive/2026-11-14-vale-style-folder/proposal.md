# Proposal

Queue row: `HKR-067` in [tasks.yaml][ledger].

## Motivation

The style `Vale` is always built in.
So hakkiri rejected every config whose packages contain a folder `styles/Vale`, even a config with no vocabulary.
Vale 66bbb85 reads its built-in style first and skips a folder of that name in silence ([manager.go][manager]).
Such a package lints in Vale and failed in hakkiri.

The operator ruled on 2026-11-13 that hakkiri skips the folder with a warning, so the other styles of the package still lint.

## Changes

- A package folder named for the built-in style `Vale` is skipped, and its rules never load or run.
- `hakkiri check` prints one warning for it.
  The warning shows the config file, the line of the package entry and the folder, and states that the built-in style `Vale` wins.
- The other styles of the package load and lint as before.
- The `project-config` spec states the skip, and the instructions, the skill and `AGENTS.md` describe it.

## Capabilities

### Modified Capabilities

- `project-config`: a package style named for a built-in style.

## Impact

- Changed: `src/config.rs`, `src/run.rs`, `tests/vocabulary_rules.rs`, `src/instructions.md`, the skill and `AGENTS.md`.
- `Project` stores the warnings of the load, which `check` prints before the warnings of the walk.
- No new dependency.

[ledger]: ../../../../tasks.yaml
[manager]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/manager.go#L523-L540
