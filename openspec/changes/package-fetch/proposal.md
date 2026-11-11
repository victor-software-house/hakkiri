# Proposal

Queue row: `HKR-006` in [tasks.yaml][ledger].  
It unblocks `HKR-019` (vocabularies) and `HKR-008` (the Vale importer's package build).

## Motivation

`check` and `test` load a package only from a local folder.
A `url` entry with its `sha256` is in the config grammar and in the [project-config spec][config], but hakkiri refuses it with "not downloaded yet".
prose-style publishes `ProseStyle.zip` as a release asset, so a project cannot pin its rules by version and hash yet.

## Changes

- A `url` entry downloads a zip, checks its SHA-256, unpacks it into a cache folder named by the hash, and loads the package's `styles` folder as a `path` entry does.
- A second run reads the cache and asks for nothing, so a cached package lints with no network.
- A download that does not match its hash, is not a zip, names a file outside the package, or holds no `styles` folder is an error that names the `url`, and nothing is cached.
- `--cache-dir`, with the variable `HAKKIRI_CACHE_DIR`, moves the cache, which defaults to the per-user cache folder.
- The `vale-differences` spec records what differs from `vale sync`.
- A patch changeset ships the feature.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `project-config`: a package `url` is fetched, checked and cached.
- `vale-differences`: the scenario for a package fetched on use names the hash and the cache.

## Impact

- New dependencies, approved by the operator (operator, 2026-11-11): `ureq` for the download, `zip` to unpack it and `sha2` for the hash, which add 1.41 MB to the musl binary.
- New: `src/package.rs`, `tests/package_fetch.rs` and its fixtures. Changed: `src/config.rs`, the CLI, `Cargo.toml`, `deny.toml`, the skill and `AGENTS.md`.

[ledger]: ../../../tasks.yaml
[config]: ../../specs/project-config/spec.md
