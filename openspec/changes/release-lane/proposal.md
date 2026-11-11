# Proposal

Queue row: `HKR-029` in [tasks.yaml][ledger].  
It is the last of three changes that ship the command line, after [`check-command`][check] and [`test-command`][test].

## Motivation

The template switch (`HKR-009`) and prose-style's CI install hakkiri through mise, so a release must exist first.
hakkiri has no release lane: its crate version is `0.0.0`, `publish = false`, and the repository has no release workflow or `.ctl/ver.yaml`.
The first release is `0.0.1`, as every new project's is.

## Changes

- A release workflow that calls verctl's reusable release workflow, and a `.ctl/ver.yaml` that declares the package with `publisher: none` and the assets.
- Each release is a tag, a GitHub Release and one tarball for each platform a consumer locks, `linux-x64` and `darwin-arm64`, each with its checksum and each built natively on a free GitHub-hosted runner (`ubuntu-latest`, `macos-latest`).
- The skill and the README name the released version, kept current by version pins.
- One changeset fragment ships `check` and `test` as `0.0.1`.
- Proof that mise installs the release on both platforms.

## Capabilities

### New Capabilities

- `release`: what a release holds, how it installs, and which texts carry its version.

### Modified Capabilities

None.

## Impact

- No code changes; no new dependency in `Cargo.toml`.
- New: `.github/workflows/release.yml`, `.ctl/ver.yaml`, `.changeset/`, and a verctl pin in `mise.toml` with its lock.
- The release run needs the organization's App secrets, which the repository already reads.

[ledger]: ../../../tasks.yaml
[check]: ../archive/2026-11-10-check-command/proposal.md
[test]: ../test-command/proposal.md
