# Proposal

Queue row: `HKR-034` in [tasks.yaml][ledger].  
It changes the lane that [`release-lane`][lane] added.

## Motivation

The `linux-x64` binary of `0.0.1` links glibc, and it starts on the NixOS machine of the install proof only because that machine enables nix-ld.
A NixOS machine without nix-ld has no loader at the path the binary asks for, so the binary does not start there.
The operator decided that the `linux-x64` asset becomes a static musl build (operator, 2026-11-11).

## Changes

- The `linux-x64` asset builds for `x86_64-unknown-linux-musl` on `ubuntu-latest` and keeps its name, so consumers change nothing.
- A release task installs `musl-tools` and the Rust target before the build, because the runner has no musl C compiler.
- The binary uses mimalloc as its global allocator under musl only, because musl's own allocator makes a parallel `check` about eight times slower (operator, 2026-11-11).
- The release spec, `AGENTS.md` and the README name the `linux-x64` binary as static.
- One patch changeset ships the change as `0.0.2`.
- Proof after the release: the binary starts with no loader, and a `check` is as fast as the glibc build.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `release`: the `linux-x64` tarball holds a static binary, and the binary keeps its speed.

## Impact

- New dependency: `mimalloc` 0.1.52, MIT, with default features off, in the `cfg(target_env = "musl")` table only. The `darwin-arm64` build and any glibc build do not link it.
- New file: `mise-tasks/release/prepare.sh`. Changed: `.ctl/ver.yaml`, `Cargo.toml`, `Cargo.lock`, `src/main.rs`.
- `darwin-arm64` is unchanged. qctl keeps its glibc build.

[ledger]: ../../../../tasks.yaml
[lane]: ../2026-11-11-release-lane/proposal.md
