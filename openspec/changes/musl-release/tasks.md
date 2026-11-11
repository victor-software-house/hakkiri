# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
The release run and the proofs after it happen after the merge, and the notes under tasks 2.1, 3.1 and 3.2 hold what they showed.  
This change follows [`release-lane`][lane], so the lane and `0.0.1` exist.

## 1. The build

- [ ] 1.1 Add `mimalloc` 0.1.52, default features off, to the `cfg(target_env = "musl")` dependencies, and make it the global allocator under that cfg in `src/main.rs`.  
  Verify `cargo tree --target aarch64-apple-darwin` and `cargo tree --target x86_64-unknown-linux-gnu` list no `mimalloc`, `cargo deny` passes, and the CI run passes.
- [ ] 1.2 Add `mise-tasks/release/prepare.sh`, and set `assets.prepare` and the `linux-x64` triple in `.ctl/ver.yaml`.  
  Verify `verctl status` reports the `linux-x64` target on `ubuntu-latest`, and a musl release build in the runner's image makes a binary that `file` reports as static.
- [ ] 1.3 Add the patch changeset that ships the musl build as `0.0.2`.  
  Verify the Version PR that follows the merge bumps `Cargo.toml` and `Cargo.lock` to `0.0.2`.

## 2. The release

- [ ] 2.1 Merge the Version PR.  
  Verify the run publishes the tag `v0.0.2` and a Release with the same two asset names, that `file` reports the `linux-x64` binary as static, and that `readelf -l` lists no interpreter.

## 3. The proofs

- [ ] 3.1 Run the released `linux-x64` binary in a root folder that holds no loader, such as an image imported from an empty archive, or on NixOS with nix-ld off.  
  Verify `--version` prints `0.0.2`, `check` flags the word of a rule in a Markdown file, and `test --coverage` passes.
- [ ] 3.2 Time the released binary against the glibc `0.0.1` binary on four CPUs over 3000 Markdown files.  
  Verify the musl binary takes at most twice as long.
- [ ] 3.3 Update `AGENTS.md` and the README with the static build.  
  Verify decision 9 of `AGENTS.md` and the README's install text name a static `linux-x64` binary.

## 4. Integration

- [ ] 4.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.

## Workflow follow-up

- Archive this change after `0.0.2`, then archive `HKR-034` with `qctl close-from-openspec`.

[lane]: ../archive/2026-11-11-release-lane/proposal.md
