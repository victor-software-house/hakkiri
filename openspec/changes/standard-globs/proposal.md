# Proposal

Queue row: `HKR-076` in [tasks.yaml][ledger].

## Motivation

hakkiri compiled each `files` and `formats` glob with globset's defaults, where `*` crosses `/`.
So `docs/*.md` matched `docs/x/a.md`, and `*.md` reached a nested file only because `*` crossed folders.
The operator ruled on 2026-11-14 that a glob follows the standard rules, whether Vale is looser or stricter.
Vale 66bbb85 reads a glob with `**` through doublestar and every other glob through gobwas with no separator, so its `*` crosses `/` and its trailing `/**` also matches the folder's own files.

## Changes

- A glob reads by the rules of gitignore: `*`, `?` and a class stay inside one folder, and `**` spans folders.
- A glob with no `/` matches the file name in any folder, so `*.md` and `README.md` still reach nested files.
- A leading `/` ties a glob to the project root, and a trailing `/` matches every file the folder holds.
- A trailing `/**` matches the files below the folder, so `tests/oracle/*/**` matches no file directly in `tests/oracle`.
- One helper, `config::standard_glob`, compiles the globs of `files` and `formats`.
- The `vale-differences` spec records where Vale matches more.

## Capabilities

### Modified Capabilities

- `project-root`: how a glob matches a path.
- `vale-differences`: where Vale's globs match more.

## Impact

- Changed: `src/config.rs`, `src/formats.rs`, `src/config/tests.rs`, `src/instructions.md` and `AGENTS.md`.
- A config with a glob such as `docs/*.md` lints fewer files: write `docs/**/*.md` for every file below `docs`.

[ledger]: ../../../tasks.yaml
