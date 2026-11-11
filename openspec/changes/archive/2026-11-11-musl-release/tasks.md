# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
The release run and the proofs after it happen after the merge, and the notes under tasks 2.1, 3.1 and 3.2 hold what they showed.  
This change follows [`release-lane`][lane], so the lane and `0.0.1` exist.

## 1. The build

- [x] 1.1 Add `mimalloc` 0.1.52, default features off, to the `cfg(target_env = "musl")` dependencies, and make it the global allocator under that cfg in `src/main.rs`.  
  Verify `cargo tree --target aarch64-apple-darwin` and `cargo tree --target x86_64-unknown-linux-gnu` list no `mimalloc`, `cargo deny` passes, and the CI run passes.  
  Evidence, 2026-11-11 (UTC): in `ubuntu:24.04` in a box, `cargo tree --locked` listed no `mimalloc` for the darwin and gnu targets, and listed 0.1.52 for the musl target.  
  `cargo deny` passed licenses, bans, sources and advisories, and [the CI run of PR 69][ci-pr69] passed.
- [x] 1.2 Add `mise-tasks/release/prepare.sh`, and set `assets.prepare` and the `linux-x64` triple in `.ctl/ver.yaml`.  
  Verify `verctl status` reports the `linux-x64` target on `ubuntu-latest`, and a musl release build in the runner's image makes a binary that `file` reports as static.  
  Evidence, 2026-11-11 (UTC): `ver status` reported `linux-x64` on `ubuntu-latest`.  
  In the same image, the build made a `static-pie linked` binary, and without `musl-tools` it failed with `failed to find tool "x86_64-linux-musl-gcc"`.
- [x] 1.3 Add the patch changeset that ships the musl build as `0.0.2`.  
  Verify the Version PR that follows the merge bumps `Cargo.toml` and `Cargo.lock` to `0.0.2`.  
  Evidence, 2026-11-11 (UTC): [Version PR 72][pr72] moved `Cargo.toml`, `Cargo.lock`, the README install line and the skill's `version:` line to `0.0.2`.  
  It consumed this changeset and the rule-setting one, and its checks passed.

## 2. The release

- [x] 2.1 Merge the Version PR.  
  Verify the run publishes the tag `v0.0.2` and a Release with the same two asset names, that `file` reports the `linux-x64` binary as static, and that `readelf -l` lists no interpreter.  
  Evidence, 2026-11-11 (UTC): [the release run][release-run] on `479e539` passed every job, with docs.rs skipped.  
  Its `linux-x64` job ran `release:prepare`, installed `musl-tools` and built in 1m40s on `ubuntu-latest`.  
  [The Release][release] holds `hakkiri_0.0.2_linux_x64.tar.gz` and `hakkiri_0.0.2_macos_arm64.tar.gz`.  
  `file` reports the binary as `static-pie linked`, and `readelf -l` lists no interpreter.

## 3. The proofs

- [x] 3.1 Run the released `linux-x64` binary in a root folder that holds no loader, such as an image imported from an empty archive, or on NixOS with nix-ld off.  
  Verify `--version` prints `0.0.2`, `check` flags the word of a rule in a Markdown file, and `test --coverage` passes.  
  Evidence, 2026-11-11 (UTC): the released tarball ran in an image with no file but the binary, on a NixOS 26.11 box.  
  `--version` printed `0.0.2`, `check` flagged the word, and `test --coverage` passed 2 cases.  
  The glibc `0.0.1` binary failed in the same image with "no such file or directory".  
  Not verified: a NixOS machine with nix-ld turned off as such, because the box enables nix-ld.
- [x] 3.2 Time the released binary against the glibc `0.0.1` binary on four CPUs over 3000 Markdown files.  
  Verify the musl binary takes at most twice as long.  
  Evidence, 2026-11-11 (UTC): three runs on four CPUs took 0.36 to 0.40 s for the musl binary and 0.38 to 0.39 s for the glibc one.  
  On one CPU they took 1.01 to 1.03 s and 1.09 to 1.12 s.  
  The `mise` pin installed `0.0.2` on a Mac, with lock checksums equal to the GitHub digests, and `check` flagged the word.
- [x] 3.3 Update `AGENTS.md` and the README with the static build.  
  Verify decision 9 of `AGENTS.md` and the README's install text name a static `linux-x64` binary.  
  Evidence, 2026-11-11 (UTC): both name it, merged with [PR 69][pr69].

## 4. Integration

- [x] 4.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.  
  Evidence, 2026-11-11 (UTC): [the CI run of PR 69][ci-pr69] passed, and so did [the CI run on main][ci-main] after the Version PR.

## Workflow follow-up

- Archive this change after `0.0.2`, then archive `HKR-034` with `qctl close-from-openspec`.

[lane]: ../2026-11-11-release-lane/proposal.md
[ci-main]: https://github.com/victor-software-house/hakkiri/actions/runs/42957727752
[ci-pr69]: https://github.com/victor-software-house/hakkiri/actions/runs/42956037775
[pr69]: https://github.com/victor-software-house/hakkiri/pull/69
[pr72]: https://github.com/victor-software-house/hakkiri/pull/72
[release]: https://github.com/victor-software-house/hakkiri/releases/tag/v0.0.2
[release-run]: https://github.com/victor-software-house/hakkiri/actions/runs/42957727646
