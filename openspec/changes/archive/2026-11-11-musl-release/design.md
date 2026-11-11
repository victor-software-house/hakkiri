# Design

## Context

The [release lane][lane] builds each asset natively on a hosted runner, and `linux-x64` links glibc.
Its install proof ran on NixOS 26.11, where the binary started because nix-ld supplies `/lib64/ld-linux-x86-64.so.2`.
A NixOS machine without nix-ld has no such loader, so a glibc binary fails there with "no such file or directory".
hakkiri links tree-sitter and three grammars, which are C, so a musl build compiles C for musl.
verctl 0.8.0 builds a target with `cargo build --release --locked --target {triple}` after a stock prepare of `rustup target add {triple}` ([README][v-readme]).

## Goals / Non-Goals

Goals:

- The `linux-x64` binary is static, so it starts with no loader.
- A `check` over many files is as fast as the glibc build.

Non-goals:

- An `aarch64` Linux asset, and any change to `darwin-arm64`.
- Cross-compiling: the build stays native on `ubuntu-latest`.

## Decisions

### 1. The linux-x64 asset is a static musl build

`.ctl/ver.yaml` gives `linux-x64` the triple `x86_64-unknown-linux-musl`, with the asset name unchanged (operator, 2026-11-11).
verctl's built-in record supplies the runner, `os` and `arch`, and a target can override one field.
Consumers still pin `github:victor-software-house/hakkiri`, and `mise` still picks `linux-x64`.

Alternative: keep glibc and tell NixOS users to enable nix-ld.
Lost: the binary then fails on a machine without it, and every glibc binary depends on the runner's glibc.

### 2. A release task installs musl-tools and the target

cc-rs looks for `x86_64-linux-musl-gcc` or `musl-gcc` when it compiles C for a musl target.
A build on `ubuntu-24.04` without `musl-tools` fails with `failed to find tool "x86_64-linux-musl-gcc"`, and a build with it succeeds.
verctl's stock prepare adds only the Rust target, and a declared `assets.prepare` replaces the stock one.
So `assets.prepare` runs `mise run release:prepare {triple}`, whose file task `mise-tasks/release/prepare.sh` adds the Rust target for every target and installs `musl-tools` for a musl one.

Alternative: point cc-rs at the runner's own `gcc`.
Lost: that compiles C against glibc headers and links it with musl's libc, and nothing would show a mismatch.

### 3. mimalloc is the global allocator under musl

musl's allocator takes a lock that threads fight over, and hakkiri lints files on every CPU.
A `check` over 3000 Markdown files (18 MB, no alert), best of two runs on a 4 vCPU machine:

| Binary | 4 CPUs | 1 CPU | Size |
|:--|:--|:--|:--|
| glibc, `0.0.1` | 0.38 s | 1.07 s | 12.5 MB |
| musl, system allocator | 3.07 s | 1.98 s | 12.7 MB |
| musl, mimalloc | 0.35 s | 0.99 s | 12.8 MB |
| musl, jemalloc | 0.36 s | 1.01 s | 12.9 MB |

The system allocator is slower with more CPUs, so a lock causes the slowdown.
`mimalloc` 0.1.52 with default features off is `[target.'cfg(target_env = "musl")'.dependencies]` in `Cargo.toml`, so the `darwin-arm64` build and any glibc build never link it (operator, 2026-11-11).
It compiles with `musl-gcc` and adds 125 KB.

Alternative: jemalloc, which is as fast.
Lost: it adds 287 KB against 125 KB.
Alternative: the system allocator.
Lost: a parallel `check` is eight times slower than the glibc build.

### 4. The proof is a root with no loader

A Docker image imported from an empty archive holds no file at all.
The glibc `0.0.1` binary mounted in it fails with "no such file or directory", and the musl binary runs `--version`, `check` and `test --coverage`.
After `0.0.2` ships, the same proof runs on the released tarball, and the timing runs on it too.

## Risks / Trade-offs

- [The first musl build of the release job runs in the release run, because PR CI builds no asset] → A box build in the runner's image, `ubuntu:24.04`, proved the same commands first. A failed run is reported with its log and not retried by hand.
- [`musl-tools` comes from the runner image's package index] → The task installs it on each run, and a failed install stops the build.

[lane]: ../2026-11-11-release-lane/design.md
[v-readme]: https://github.com/victor-software-house/verctl/blob/bc30ea6f0924edd8554ef5444280d9ca52dd1851/README.md
