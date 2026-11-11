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
- A package file holds `files`, `level`, `vocab` and `packages`.
  Any other key is an error that names the key, the line and the package.
- A package's `packages` are fetched like the project's, each pinned by `url` and `sha256`.
  A chain more than 10 packages deep is an error.
- The settings of a package that ships a rule `off` leave it off until the project turns it on.
- The `vale-differences` spec records where this differs from how Vale 3.26.0 merges a package's `.vale.ini`.
- A patch changeset ships the feature.

## Decisions

The operator ruled on 2026-11-11, and the design records each with its alternatives.

1. **The file** is `.ctl/hakkiri.yaml` inside the package (operator, 2026-11-11).
2. **Styles** add up under the same glob text and the project's list replaces the package's under another glob that matches the same files, as in Vale, with no warning (operator, 2026-11-11).
3. **Nested packages** are fetched, as Vale does, pinned by `url` and `sha256`, with a depth limit (operator, 2026-11-11).

## Capabilities

### New Capabilities

- `package-settings`: where a package's settings file sits, the packages a package lists, and its vocabularies.

### Modified Capabilities

- `project-config`: a package's settings file is read and merged under the project's config.
- `vale-differences`: the scenarios for what differs from `vale sync` and Vale's merge.

## Impact

- Changed: `src/config.rs` (the layering), the `--help` text, `src/instructions.md`, the skill, `AGENTS.md`.
  New: `tests/package_settings.rs`.
- No new dependency.
- The ProseStyle package gains `.ctl/hakkiri.yaml` and loses `.vale.ini`, in the prose-style repository (`HKR-008`).

[ledger]: ../../../tasks.yaml
