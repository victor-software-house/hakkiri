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
- [x] 1.3 Place a fault about one item of a list, a `styles` name, a `vocab` name or an ignore pattern, at that item.  
  Verify the three tests for a later item fail before the change and pass after it.
  Evidence, 2026-11-11 (UTC): against the previous `config.rs` in a box the three tests fail, and they pass with the change.  
  `format`, `lint`, the full suite, `specs`, `specs:budget`, `lint:md`, `lint:typos`, `lint:prose`, `check:versions` and `check:repo` pass.
- [x] 1.4 Place a fault about a key itself with ctl-core's `Input::place_key`, from ctl-core 0.7.2.  
  Verify the two tests for a key with a block value fail before the change and pass after it.
  Evidence, 2026-11-11 (UTC): against the previous `config.rs` in a box the two tests fail, and they pass with the change.  
  `format`, `lint`, the full suite, `specs`, `specs:budget`, `lint:md`, `lint:typos`, `lint:prose`, `check:versions` and `check:repo` pass.  
  The box's crates index lagged on ctl-core 0.7.2 for a few minutes, and the lockfile update passed on a retry.

## 2. Integration

- [x] 2.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.  
  Evidence, 2026-11-12 (UTC): PRs 114, 116 and 118 merged, and CI on main at 930ecee passed: `Build and test`, `Static checks` and `Verify`.
