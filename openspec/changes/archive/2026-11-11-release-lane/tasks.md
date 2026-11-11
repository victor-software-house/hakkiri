# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
The release run and the install proof happen after the merge, and the notes under tasks 2.1 and 3.1 hold what they showed.  
This change follows [`check-command`][check] and [`test-command`][test], so the binary and the skill exist.

## 1. The lane

- [x] 1.1 Add `.github/workflows/release.yml`, which calls verctl's reusable workflow at the commit of its latest release, and pin that verctl in `mise.toml` with its `ver` task include and a refreshed lock.  
  Verify `lint:workflow-policy` and `lint:workflows` pass, and `check:versions` passes in CI.  
  Evidence, 2026-11-11 (UTC): [PR 65][pr65] pins verctl 0.8.0, the latest release, and its CI run passes the Static checks job that holds those three tasks ([CI run][ci-pr65]).  
  The pre-push lockfile hook also required `verctl` in `mise.ci.lock` and `mise.dev.lock`, because the release jobs run with `MISE_ENV=ci`.
- [x] 1.2 Add `.ctl/ver.yaml` with `publisher: none`, the `prepare.after` that regenerates `Cargo.lock`, the two assets (built natively on `macos-latest` and `ubuntu-latest`, with no `runners` entry), the two patterns and their pins, and the `changelog` block.  
  Verify `verctl status` in CI reports one package at `0.0.0` and no pattern error.  
  Evidence, 2026-11-11 (UTC): the Plan job of [the release run after PR 65][release-65] ran `ver status` and passed.  
  verctl 0.8.0 refuses a pattern with an open side, so both patterns carry `whole_line: true`, and the README pin is the whole line `mise use github:victor-software-house/hakkiri@{version}`.
- [x] 1.3 Add the changeset fragment that ships `check` and `test` as a patch.  
  Verify the Version PR that the first push after the merge opens bumps `Cargo.toml` and `Cargo.lock` to `0.0.1`.  
  Evidence, 2026-11-11 (UTC): [Version PR 66][pr66] moved `Cargo.toml` and `Cargo.lock` to `0.0.1`, the README install line and the skill's `version:` line with them, and wrote a `CHANGELOG.md` entry.

## 2. The first release

- [x] 2.1 Merge the Version PR.  
  Verify the run publishes the tag `v0.0.1` and a Release with two tarballs and their checksums, and that crates.io has no `hakkiri` crate.  
  Evidence, 2026-11-11 (UTC): [the release run][release-run] on the merge commit `c3f3b38` passed its Plan, Publish, `darwin-arm64`, `linux-x64` and Version PR jobs, and skipped docs.rs.  
  [The Release][release] holds `hakkiri_0.0.1_linux_x64.tar.gz` and `hakkiri_0.0.1_macos_arm64.tar.gz` on the tag `v0.0.1`, which names `c3f3b38`.  
  It holds no `.sha256` file: GitHub records each tarball's SHA-256 digest, which `mise lock` reads, so design decision 4 and the release spec hold that.  
  The crates.io API answered 404 for `hakkiri`.

## 3. The install proof

- [x] 3.1 In an empty folder on a Mac and on Linux x86-64, including a NixOS machine, pin `github:victor-software-house/hakkiri` at `0.0.1`, run `mise lock` for both platforms, then `mise install`.  
  Verify `hakkiri --version` prints `0.0.1` on both, and `hakkiri check` lints a Markdown file on both, and record whether the glibc binary starts on the NixOS machine.  
  A binary that does not start there is reported as a finding, and the lane is not changed to work around it.  
  Evidence, 2026-11-11 (UTC): an empty folder on a Mac (arm64) and an empty folder on NixOS 26.11 (x86-64, glibc 2.44) each ran `mise use github:victor-software-house/hakkiri@0.0.1` and `mise lock --platform linux-x64,macos-arm64`.  
  Each lock holds checksums that equal the digests of the two tarballs.  
  `hakkiri --version` printed `0.0.1` on both, `hakkiri check` flagged `very` in a Markdown file, and `hakkiri test --coverage` passed 2 cases on both.  
  The glibc binary starts on that NixOS machine because it enables nix-ld, which provides `/lib64/ld-linux-x86-64.so.2`.  
  The binary needs `libgcc_s.so.1`, `libm.so.6` and `libc.so.6`, and glibc 2.35 at most.  
  **Finding:** a NixOS machine without nix-ld has no loader at that path, and this binary does not start there.  
  Not verified: a NixOS machine without nix-ld, and `ubuntu-latest` moving to Ubuntu 26, which raises the glibc the next build needs.
- [x] 3.2 Update `AGENTS.md` with the release rule: a behaviour change adds a fragment, and the Version PR is the release.  
  Verify `AGENTS.md` holds the rule.  
  Evidence, 2026-11-11 (UTC): decision 9 of `AGENTS.md` holds it, merged with [PR 65][pr65].

## 4. Integration

- [x] 4.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec and the Template check.  
  Evidence, 2026-11-11 (UTC): [PR 65][pr65] passed the CI run's Verify, Static checks and Build and test jobs, and its Template check, secrets scan, CodeQL and Kody.

## Workflow follow-up

- Archive this change after the first release, then archive `HKR-029` with `qctl close-from-openspec`.

[check]: ../2026-11-10-check-command/proposal.md
[test]: ../2026-11-10-test-command/proposal.md
[ci-pr65]: https://github.com/victor-software-house/hakkiri/actions/runs/42950347129
[pr65]: https://github.com/victor-software-house/hakkiri/pull/65
[pr66]: https://github.com/victor-software-house/hakkiri/pull/66
[release]: https://github.com/victor-software-house/hakkiri/releases/tag/v0.0.1
[release-65]: https://github.com/victor-software-house/hakkiri/actions/runs/42951020311
[release-run]: https://github.com/victor-software-house/hakkiri/actions/runs/42951830393
