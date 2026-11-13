# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The tag

- [x] 1.1 Move the ctl-core pin to 0.7.5 and read the glob settings and the top-level `code_span_size` through `ctl_core::input::Tagged` in `src/config.rs`.  
  Refuse a tag other than `!override`, and `!override` in a package's settings file, on the line of its key.  
  Verify the tests for the scenarios "A package that forces" and "Another tag" pass.  
  Evidence: `a_package_that_forces_a_value_is_an_error_naming_its_line` and `a_tag_other_than_override_is_an_error_naming_its_line` in `tests/package_settings.rs` pass in a 4x8 box, with the whole suite.

## 2. The cascade

- [x] 2.1 Rank the globs once at load by specificity, origin and written place, as `design.md` section 1 and 2 say, and fold the forced values after the others in `plan` and `build_variants`.  
  Verify a unit test of the specificity measure over the table of `design.md` section 1, and a test for each scenario of the `project-config` delta.  
  Evidence: `src/config/tests.rs` and the new tests of `tests/rule_setting.rs`, `tests/package_settings.rs` and `tests/code_span_size.rs` pass in the box.
- [x] 2.2 Rewrite the tests whose scenarios changed: "A level that stays", "An empty list of styles", "Another glob" and the `rule-parameters` scenario "An empty styles list".  
  Verify they pass with the new expectations.  
  Evidence: the four tests in `tests/rule_setting.rs`, `tests/package_settings.rs` and `tests/rule_parameters.rs` pass in the box.

## 3. The surfaces

- [x] 3.1 Update `src/instructions.md`, the skill and `AGENTS.md`, add the `vale-differences` delta, and add the minor changeset.  
  Verify `operator_docs` and `openspec validate --all --strict` pass.  
  Evidence: `operator_docs` passes in the box, and the `specs` task validates every change.

## 4. Integration

- [ ] 4.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
