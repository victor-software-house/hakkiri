# Proposal

Queue row: `HKR-067` in [tasks.yaml][ledger].

## Motivation

The style `Vale` is always built in, so hakkiri refused every config whose packages hold a folder `styles/Vale`, even a config that names no vocabulary.
Vale 66bbb85 reads its built-in style first and passes over a folder of that name without a word ([manager.go][manager]), so such a package lints in Vale and failed in hakkiri.
The operator ruled on 2026-11-13 that hakkiri skips the folder with a warning, so the other styles of the package still lint.

## Changes

- A package folder named for the built-in style `Vale` is skipped, and its rules never load or run.
- `hakkiri check` prints one warning for it, which names the config file and line of the package entry and the folder, and says the built-in style `Vale` wins.
- The other styles of the package load and lint as before.
- The `project-config` spec states the skip, and the instructions, the skill and `AGENTS.md` describe it.

## Capabilities

### Modified Capabilities

- `project-config`: a package style named for a built-in style.

## Impact

- Changed: `src/config.rs`, `src/run.rs`, `tests/vocabulary_rules.rs`, `src/instructions.md`, the skill and `AGENTS.md`.
- `Project` holds the warnings of the load, which `check` prints before the warnings of the walk.
- No new dependency.

[ledger]: ../../../../tasks.yaml
[manager]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/manager.go#L523-L540
