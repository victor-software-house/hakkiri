# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The load

- [x] 1.1 Skip a package folder named for the built-in style with a warning in `src/config.rs`, and print the load warnings in `src/run.rs`.  
  Verify `a_package_style_named_vale_is_skipped_with_a_warning` in `tests/vocabulary_rules.rs` passes.  
  Evidence, 2026-11-14 (UTC): [pull request 149](https://github.com/victor-software-house/hakkiri/pull/149) adds the skip, and its test `a_package_style_named_vale_is_skipped_with_a_warning` passes in CI.

## 2. The surfaces

- [x] 2.1 Update `src/instructions.md`, the skill and `AGENTS.md`, add the `project-config` delta, and add the patch changeset.  
  Verify `operator_docs` and `openspec validate --all --strict` pass.  
  Evidence, 2026-11-14 (UTC): [pull request 149](https://github.com/victor-software-house/hakkiri/pull/149) adds the surfaces, the `project-config` delta and the changeset.

## 3. Integration

- [x] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.  
  Evidence, 2026-11-14 (UTC): The CI of [pull request 149](https://github.com/victor-software-house/hakkiri/pull/149) passed, and it merged through the merge queue.
