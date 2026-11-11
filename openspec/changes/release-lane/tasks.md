# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
The release run and the install proof happen after the merge, and the notes under tasks 2.1 and 3.1 hold what they showed.  
This change follows [`check-command`][check] and [`test-command`][test], so the binary and the skill exist.

## 1. The lane

- [ ] 1.1 Add `.github/workflows/release.yml`, which calls verctl's reusable workflow at the commit of its latest release, and pin that verctl in `mise.toml` with its `ver` task include and a refreshed lock.  
  Verify `lint:workflow-policy` and `lint:workflows` pass, and `check:versions` passes in CI.
- [ ] 1.2 Add `.ctl/ver.yaml` with `publisher: none`, the `prepare.after` that regenerates `Cargo.lock`, the two assets (built natively on `macos-latest` and `ubuntu-latest`, with no `runners` entry), the two patterns and their pins, and the `changelog` block.  
  Verify `verctl status` in CI reports one package at `0.0.0` and no pattern error.
- [ ] 1.3 Add the changeset fragment that ships `check` and `test` as a patch.  
  Verify the Version PR that the first push after the merge opens bumps `Cargo.toml` and `Cargo.lock` to `0.0.1`.

## 2. The first release

- [ ] 2.1 Merge the Version PR.  
  Verify the run publishes the tag `v0.0.1` and a Release with two tarballs and their checksums, and that crates.io has no `hakkiri` crate.

## 3. The install proof

- [ ] 3.1 In an empty folder on a Mac and on Linux x86-64, including a NixOS machine, pin `github:victor-software-house/hakkiri` at `0.0.1`, run `mise lock` for both platforms, then `mise install`.  
  Verify `hakkiri --version` prints `0.0.1` on both, and `hakkiri check` lints a Markdown file on both, and record whether the glibc binary starts on the NixOS machine.  
  A binary that does not start there is reported as a finding, and the lane is not changed to work around it.
- [ ] 3.2 Update `AGENTS.md` with the release rule: a behaviour change adds a fragment, and the Version PR is the release.  
  Verify `AGENTS.md` holds the rule.

## 4. Integration

- [ ] 4.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec and the Template check.

## Workflow follow-up

- Archive this change after the first release, then archive `HKR-029` with `qctl close-from-openspec`.

[check]: ../archive/2026-11-10-check-command/proposal.md
[test]: ../test-command/proposal.md
