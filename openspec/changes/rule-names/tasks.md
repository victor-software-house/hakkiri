# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. Probe Vale

- [x] 1.1 Build Vale `66bbb85` in a throwaway Linux box, and run it over styles that hold the cases below.  
  The cases are a dotted file name, a nested rule at two levels, two files with one name, a bracket name, a `*.test.yml` file, and folders that start with `.` and `_`.  
  Verify the rule names, the loaded rules and the errors match the scenarios of the `rule-loading` delta, and record what differs in `vale-differences`.
  - 2026-11-11 (UTC), `Terms.custom.yml` gave `Team.Terms`, and `Sub/Deep/Baz.yml` gave `Team.Sub.Deep.Baz`.  
    `Terms.yml` with `Terms.custom.yml`, and a bracket name, each failed with error E100 in an enabled style.  
    Neither failed when no `BasedOnStyles` named the style, and Vale linted with exit 0.  
    `vale test --coverage` named a nested rule `Team.Sub.Deep.Bar` and left a rule under `_draft` out of the coverage.

## 2. Name and load

- [x] 2.1 Cut a rule name at the first dot, join the folders under the style, refuse a bracket, and walk every folder under a style as Vale does, in `src/rule.rs`.  
  Verify the `check` tests for a dotted name, a nested rule and a bracket fail on the old loader and pass on the new one.
  - 2026-11-11 (UTC), the three tests failed on the loader of [de3e392][c-base] and pass in [a1e07da][c-code], in a throwaway Linux box.
- [x] 2.2 Refuse two files that give one rule name, with an error that names the rule and both files.  
  Verify the clash test fails on the old loader and passes on the new one.
  - 2026-11-11 (UTC), the same box run shows it failing on the old loader and passing on [a1e07da][c-code].
- [x] 2.3 Name a rule in `test` and in `test --coverage` through the same function, with the style found from the packages of the config.  
  Verify the dotted and nested coverage tests fail on the old loader and pass on the new one.
  - 2026-11-11 (UTC), both `test_command` tests failed on the old loader and pass on [a1e07da][c-code].  
    The dotted test now holds `absent: Team.Foo.v2`, since `contains: Team.Foo` also matched the old name.
- [x] 2.4 Refuse a name clash or a bracket only for a style a glob lists, as Vale does for an enabled style, and lint an unused style with no error.  
  Verify the `check` tests for an unused style with a clash and with a bracket fail on the loader of [3e368a0][c-prior] and pass on the new one, and the tests for a listed style still pass.
  - 2026-11-11 (UTC), both unused-style tests failed with exit status 2 on that loader, and pass in [eae1653][c-lazy], in a throwaway Linux box.

## 3. Record

- [x] 3.1 Update `src/instructions.md`, the `rule-loading` spec, and the unarchived `test-command` change.  
  Verify `openspec validate --all --strict` and the budget check pass.
  - 2026-11-11 (UTC), both pass on the Mac for the docs of [eeafcb0][c-docs] with two sentences split.

## 4. Integration

- [ ] 4.1 Verify the pull request's CI passes: build, lint, tests, prose lint, OpenSpec and the Template check.

## Workflow follow-up

- Archive this change after the pull request merges, then archive `HKR-032` with `qctl close-from-openspec`.

[c-base]: https://github.com/victor-software-house/hakkiri/commit/de3e392ed578ede50c86d753ec7c1e6114dbc9ad
[c-code]: https://github.com/victor-software-house/hakkiri/commit/a1e07daf0072c127692d6f8979263bbde2e7b338
[c-prior]: https://github.com/victor-software-house/hakkiri/commit/3e368a0cf0793030a21d33e10c884956f7a72bd1
[c-lazy]: https://github.com/victor-software-house/hakkiri/commit/eae16539ebdb1c6b9b7e1123a2ac57ac951d17a2
[c-docs]: https://github.com/victor-software-house/hakkiri/commit/eeafcb035627fee88eea11feb2232f123292868a
