# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The tag

- [ ] 1.1 Move the ctl-core pin to 0.7.5 and read the glob settings and the top-level `code_span_size` through `ctl_core::input::Tagged` in `src/config.rs`.  
  Refuse a tag other than `!override`, and `!override` in a package's settings file, on the line of its key.  
  Verify the tests for the scenarios "A package that forces" and "Another tag" pass.

## 2. The cascade

- [ ] 2.1 Rank the globs once at load by specificity, origin and written place, as `design.md` section 1 and 2 say, and fold the forced values after the others in `plan` and `build_variants`.  
  Verify a unit test of the specificity measure over the table of `design.md` section 1, and a test for each scenario of the `project-config` delta.
- [ ] 2.2 Rewrite the tests whose scenarios changed: "A level that stays", "An empty list of styles", "Another glob" and the `rule-parameters` scenario "An empty styles list".  
  Verify they pass with the new expectations.

## 3. The surfaces

- [ ] 3.1 Update `src/instructions.md`, the skill and `AGENTS.md`, add the `vale-differences` delta, and add the minor changeset.  
  Verify `operator_docs` and `openspec validate --all --strict` pass.

## 4. Integration

- [ ] 4.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
