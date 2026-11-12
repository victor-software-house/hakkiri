# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The line of a key

- [x] 1.1 Take ctl-core 0.7.1, and place each fault of the config through `Input::place`.  
  Verify the two tests for globs and settings that hold special characters and for a config in flow style fail before the change and pass after it.  
  Evidence, 2026-11-11 (UTC): against the old `config.rs` in a box both tests fail. The first glob's fault names line 7, the line of a later glob that sets the rule, and the flow-style fault names no line. Both pass with the change.  
  The rest of the suite passes unchanged, so no other config error moved its line.
- [x] 1.2 Add the `project-config` delta and the patch changeset.  
  Verify the OpenSpec validation passes.  
  Evidence, 2026-11-11 (UTC): `format`, `lint`, the full suite, `specs`, `specs:budget`, `lint:md`, `lint:typos`, `lint:prose`, `check:versions` and `check:repo` pass in a box.  
  The box's crates mirror lacked ctl-layout 0.7.1, so the lockfile update and the run read crates.io directly.

## 2. Integration

- [ ] 2.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
