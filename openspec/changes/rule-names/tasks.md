# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. Probe Vale

- [ ] 1.1 Build Vale `66bbb85` in a throwaway Linux box, and run it over a style with a dotted file name, a nested rule at two levels, two files with one name, a bracket name, a `*.test.yml` file, and folders that start with `.` and `_`.  
  Verify the rule names, the loaded rules and the errors match the scenarios of the `rule-loading` delta, and record what differs in `vale-differences`.

## 2. Name and load

- [ ] 2.1 Cut a rule name at the first dot, join the folders under the style, refuse a bracket, and walk every folder under a style as Vale does, in `src/rule.rs`.  
  Verify the `check` tests for a dotted name, a nested rule and a bracket fail on the old loader and pass on the new one.
- [ ] 2.2 Refuse two files that give one rule name, with an error that names the rule and both files.  
  Verify the clash test fails on the old loader and passes on the new one.
- [ ] 2.3 Name a rule in `test` and in `test --coverage` through the same function, with the style found from the packages of the config.  
  Verify the nested coverage test fails on the old loader and passes on the new one.

## 3. Record

- [ ] 3.1 Update `src/instructions.md`, the `rule-loading` spec, the unarchived `test-command` change and the `vale-differences` spec.  
  Verify `openspec validate --all --strict` and the budget check pass.

## 4. Integration

- [ ] 4.1 Verify the pull request's CI passes: build, lint, tests, prose lint, OpenSpec and the Template check.

## Workflow follow-up

- Archive this change after the pull request merges, then archive `HKR-032` with `qctl close-from-openspec`.
