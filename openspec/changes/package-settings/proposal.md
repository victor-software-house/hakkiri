# Proposal

Queue row: `HKR-040` in [tasks.yaml][ledger].  
It blocks `HKR-008` (ProseStyle on hakkiri) and waits for `HKR-019` (vocabularies).

## Motivation

A Vale package ships a `.vale.ini` that Vale merges under the project's own.
ProseStyle uses it to turn its rules on for `*.md`, `*.mdx` and `*.rs`, to switch a few rules off in Rust files, and to name its vocabulary.
hakkiri keeps that file in the cache and reads none of it, so each project would have to copy those settings.

The operator decided that a package ships a settings file in hakkiri's own YAML format, that hakkiri merges it under the project's config, and that the project overrides the package (operator, 2026-11-11).
`HKR-008` also needs it: the 14 opt-in rules of the ProseStyle port ship off through the package settings.

## Changes

- A package may hold a settings file in the format of `.ctl/hakkiri.yaml`.
  hakkiri reads it for each loaded package and applies it before the project's own settings.
- The project wins over a package, and a later package wins over an earlier one.
- A package file holds `files`, `level` and `vocab`.
  `packages` and any other key is an error that names the key, the line and the file.
- The settings of a package that ships a rule `off` leave it off until the project turns it on.
- The `vale-differences` spec records where this differs from how Vale 3.26.0 merges a package's `.vale.ini`.
- A patch changeset ships the feature.

## Decisions for the operator

The design lists each with its candidates.
The specs follow the first candidate, which is provisional until the operator rules.

1. **The file's name and place in the package.** `.ctl/hakkiri.yaml`, `hakkiri.yaml` or `settings.yaml`.
2. **A project `styles` list under the glob the package also writes.** Add to the package's list as Vale does, or replace it.
3. **A package that lists `packages`.** Refuse it now, or install the nested packages as Vale does.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `project-config`: a package's settings file is read and merged under the project's config.
- `vale-differences`: the scenarios for what differs from `vale sync` and Vale's merge.

## Impact

- Changed: `src/config.rs` (the layering), the `--help` text, `src/instructions.md`, the skill, `AGENTS.md`.
  New: `tests/package_settings.rs`.
- No new dependency.
- The ProseStyle package gains `.ctl/hakkiri.yaml` and loses `.vale.ini`, in the prose-style repository (`HKR-008`).

[ledger]: ../../../tasks.yaml
