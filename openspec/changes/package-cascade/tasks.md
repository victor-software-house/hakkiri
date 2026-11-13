# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The tag

- [x] 1.1 Move the ctl-core pin to 0.7.5 and read the glob settings and the top-level `code_span_size` through `ctl_core::input::Tagged` in `src/config.rs`.  
  Refuse a tag other than `!override`, and `!override` in a package's settings file, on the line of its key.  
  Verify the tests for the scenarios "A package that forces" and "Another tag" pass.  
  Evidence: `a_package_that_forces_a_value_is_an_error_naming_its_line` and `a_tag_other_than_override_is_an_error_naming_its_line` in `tests/package_settings.rs` pass in a 4x8 box, with the whole suite.

## 2. Origin and the forced pass

- [x] 2.1 Rank a package's globs below the project's at equal specificity, and the top level below every glob.  
  Fold the forced values after the others in `plan` and `build_variants`, and let a forced list replace the unforced ones.  
  Verify a test for each scenario of the `project-config` delta of this change.  
  Evidence: the origin, forced and list tests of `tests/package_settings.rs` and `tests/code_span_size.rs` pass in a 4x8 box, with the whole suite.

## 3. Integration

- [ ] 3.1 Verify the pull request's CI passes, the same run as `settings-cascade`.
