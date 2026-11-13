# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The cascade

- [x] 1.1 Rank the globs once at load by specificity, then origin, then written place, as `design.md` sections 1 and 2 say, and fold them in rank order in `plan` and `build_variants`.  
  Verify a unit test of the specificity measure over the table of `design.md` section 1, and a test for each scenario of the `project-config` delta.  
  Evidence: `src/config/tests.rs` and the new tests of `tests/rule_setting.rs`, `tests/package_settings.rs` and `tests/code_span_size.rs` pass in the box.
- [x] 1.2 Rewrite the tests whose scenarios changed: "A level that stays", "An empty list of styles", "Another glob" and the `rule-parameters` scenario "An empty styles list".  
  Verify they pass with the new expectations.  
  Evidence: the four tests in `tests/rule_setting.rs`, `tests/package_settings.rs` and `tests/rule_parameters.rs` pass in the box.

## 2. The surfaces

- [x] 2.1 Update `src/instructions.md`, the skill and `AGENTS.md`, add the `vale-differences` delta, and add the minor changeset.  
  Verify `operator_docs` and `openspec validate --all --strict` pass.  
  Evidence: `operator_docs` passes in the box, and the `specs` task validates every change.

## 3. Integration

- [ ] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
