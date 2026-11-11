# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
This change adds to the binary that [`check-command`][check] creates.

## 1. Cases in rule files

- [ ] 1.1 Add `tests` to every check type of the rule schema, read as the cases of design decision 2.  
  Verify a rule with `tests` loads, `check` runs no case, and a malformed `tests` is refused with its line.
- [ ] 1.2 Read `*.test.yml` files and the `tests` of rule files into one list of cases, with the checks of decision 2.  
  Verify the scenarios for a case that asserts nothing, a duplicate name and an unknown field pass.

## 2. Running a case

- [ ] 2.1 Render a case's alerts as `line:column:Rule:message` lines and compare `want`, `contains` and `absent`.  
  Verify the scenario for contains and want passes, and that ProseStyle's `want: ''` reads as no alerts.
- [ ] 2.2 Load one rule alone for an isolated case, and lint a non-isolated case as `input.<format>` through the config.  
  Verify the scenarios for a rule turned off in the config and for a case through the config pass.

## 3. The command

- [ ] 3.1 Find cases as design decision 5 says, and add `hakkiri test` with `--coverage`.  
  Verify the scenarios for a rule and a test file, YAML that is not a rule, no case, and an uncovered rule pass.
- [ ] 3.2 Build the result that ctl-core renders as the report and as JSON, with exit codes 0, 1 and 2.  
  Verify the scenarios for a failed case, a case that cannot run and JSON with colour forced pass, through the shared helper that clears the colour variables.
- [ ] 3.3 Run `hakkiri test --coverage` over a fixture style folder of three rules that carry cases in the form ProseStyle uses, including `want: ''` and `format: rs`.  
  Verify a test passes every case and names no rule, and ProseStyle's own cases are left to `HKR-008`.

## 4. The bundle

- [ ] 4.1 Add `test` to `src/instructions.md` and `skills/hakkiri/SKILL.md`, and the three differences to the `vale-differences` spec.  
  Verify the instructions test passes, and `openspec validate --all --strict` and `qctl check` pass.

## 5. Integration

- [ ] 5.1 Verify the pull request's CI passes: build, lint, tests, docs, deny, prose lint and OpenSpec.

## Workflow follow-up

- Archive this change after the pull request merges, then archive `HKR-028` with `qctl close-from-openspec`.

[check]: ../archive/2026-11-10-check-command/proposal.md
