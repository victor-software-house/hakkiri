# Proposal

Queue row: `HKR-044` in [tasks.yaml][ledger].
It changes how [check-command][check] finds files, and adds three keys to [project-config][config].

## Motivation

A walk reads every file under a folder that a glob selects, including a build folder that git ignores.
A prose lint task passes every tracked file to Vale, because Vale walks the folders a build writes, and with this change it can run a bare `hakkiri check`.
A file named on the command line that no glob selects is skipped with no message, so a typo in a path or a glob looks like a clean run.
`--fail-level` is a flag and a variable but no config key, though every option reads the flag, then the variable, then the config, then the default (operator, 2026-11-11).

## Changes

- A walk skips what `.gitignore`, `.git/info/exclude` and the global git excludes ignore, in a git repository, unless `gitignore` is false (`--no-gitignore`, `--gitignore=false`, `HAKKIRI_GITIGNORE`).
- A named file is never filtered by git.
- A named file that no glob selects draws a warning, and `on_unmatched` (`--on-unmatched`, `HAKKIRI_ON_UNMATCHED`) takes `warn`, `error` or `ignore`.
- `fail_level`, `gitignore` and `on_unmatched` are config keys, each resolved as flag, then variable, then config, then default.
- A package's settings file that holds one of the three keys is an error.
- The `vale-differences` spec lists the walk as an intended difference.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `check-command`: the walk skips what git ignores, and a named file that no glob selects draws a warning.
- `project-config`: three run options are keys.
- `package-settings`: a package's file does not set the run options.
- `vale-differences`: a walk and a named file differ from Vale's.

## Impact

- Changed: `src/files.rs`, `src/run.rs`, `src/cli.rs`, `src/config.rs`, `src/instructions.md`, the skill, `AGENTS.md` and `README.md`.
- New dependency: the `ignore` crate, 0.4.34 (authorized by the operator on 2026-11-11), the walk of ripgrep.

[ledger]: ../../../../tasks.yaml
[check]: ../../../specs/check-command/spec.md
[config]: ../../../specs/project-config/spec.md
