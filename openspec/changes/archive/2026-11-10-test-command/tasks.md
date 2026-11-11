# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
This change adds to the binary that [`check-command`][check] creates.

## 1. Cases in rule files

- [x] 1.1 Add `tests` to every check type of the rule schema, read as the cases of design decision 2.  
  Verify a rule with `tests` loads, `check` runs no case, and a malformed `tests` is refused with its line.  
  Evidence, 2026-11-11 (UTC): `a_malformed_case_in_a_rule_is_refused_at_load` and `check_loads_the_cases_and_runs_none` pass in [the main CI run][ci-main].
- [x] 1.2 Read `*.test.yml` files and the `tests` of rule files into one list of cases, with the checks of decision 2.  
  Verify the scenarios for a case that asserts nothing, a duplicate name and an unknown field pass.  
  Evidence, 2026-11-11 (UTC): `a_case_that_asserts_nothing_does_not_run` and `a_duplicate_name_and_an_unknown_field_are_refused` pass in [the main CI run][ci-main].

## 2. Running a case

- [x] 2.1 Render a case's alerts as `line:column:Rule:message` lines and compare `want`, `contains` and `absent`.  
  Verify the scenario for contains and want passes, and that ProseStyle's `want: ''` reads as no alerts.  
  Evidence, 2026-11-11 (UTC): `a_want_that_differs_prints_a_diff` and `a_style_folder_of_three_rules_covers_every_rule`, whose rules hold `want: ''`, pass in [the main CI run][ci-main].
- [x] 2.2 Load one rule alone for an isolated case, and lint a non-isolated case as `input.<format>` through the config.  
  Verify the scenarios for a rule turned off in the config and for a case through the config pass.  
  Evidence, 2026-11-11 (UTC): `an_isolated_case_ignores_the_config_and_another_follows_it` and `a_case_through_the_config_lints_with_its_styles` pass in [the main CI run][ci-main].

## 3. The command

- [x] 3.1 Find cases as design decision 5 says, and add `hakkiri test` with `--coverage`.  
  Verify the scenarios for a rule and a test file, YAML that is not a rule, no case, and an uncovered rule pass.  
  Evidence, 2026-11-11 (UTC): `a_rule_and_a_test_file_run_five_cases`, `yaml_that_is_not_a_rule_is_not_read`, `no_case_is_an_error` and `coverage_names_a_rule_no_case_fires` pass in [the main CI run][ci-main].
- [x] 3.2 Build the result that ctl-core renders as the report and as JSON, with exit codes 0, 1 and 2.  
  Verify the scenarios for a failed case, a case that cannot run and JSON with colour forced pass, through the shared helper that clears the colour variables.  
  Evidence, 2026-11-11 (UTC): `a_failed_case_prints_its_file_name_and_reason`, `a_case_that_cannot_run_is_reported_with_the_others` and `json_holds_no_escape_byte_under_forced_colour` pass in [the main CI run][ci-main].
- [x] 3.3 Run `hakkiri test --coverage` over a fixture style folder of three rules that carry cases in the form ProseStyle uses, including `want: ''` and `format: rs`.  
  Verify a test passes every case and names no rule, and ProseStyle's own cases are left to `HKR-008`.  
  Evidence, 2026-11-11 (UTC): `a_style_folder_of_three_rules_covers_every_rule` passes in [the main CI run][ci-main].  
  The fixture holds six cases, with `want: ''` and `format: rs`, and ProseStyle's own cases are left to `HKR-008`.
- [x] 3.4 Name a rule from its absolute path with symlinks left alone, and read its text from that path.  
  Verify the scenarios of `test-rule-names` pass.  
  Evidence, 2026-11-11 (UTC): `a_rule_file_is_named_from_inside_its_style_folder`, `a_symlinked_style_folder_is_named_by_its_link`, `a_rule_path_through_a_symlink_and_back_out_is_read_from_the_folded_path` and `a_yaml_rule_file_is_neither_read_nor_covered` pass in [the main CI run][ci-main].  
  [PR 60][pr60] later changed which name a dotted file name and a nested folder give, so the spec holds that rule in place of the one [PR 59][pr59] first wrote.

## 4. The bundle

- [x] 4.1 Add `test` to `src/instructions.md` and `skills/hakkiri/SKILL.md`, and the three differences to the `vale-differences` spec.  
  Verify the instructions test passes, and `openspec validate --all --strict` and `qctl check` pass.  
  Evidence, 2026-11-11 (UTC): The 3 tests of `operator_docs` pass, and the `specs` task reports 22 of 22 specs, in [the main CI run][ci-main].  
  `qctl check` ran in the local gate, not in the CI log.

## 5. Integration

- [x] 5.1 Verify the pull request's CI passes: build, lint, tests, docs, deny, prose lint and OpenSpec.  
  Evidence, 2026-11-11 (UTC): [The main CI run][ci-main] on [4979c2a][main] passed Verify (Build and test, Static checks), with 28 tests in `test_command`, `cargo deny` and `cargo-machete` clean.  
  [PR 57][pr57] and [PR 59][pr59] passed CI before they merged.

## Workflow follow-up

- Archive this change after the pull request merges, then archive `HKR-028` with `qctl close-from-openspec`.

[check]: ../2026-11-10-check-command/proposal.md
[pr57]: https://github.com/victor-software-house/hakkiri/pull/57
[pr59]: https://github.com/victor-software-house/hakkiri/pull/59
[pr60]: https://github.com/victor-software-house/hakkiri/pull/60
[ci-main]: https://github.com/victor-software-house/hakkiri/actions/runs/42948574268
[main]: https://github.com/victor-software-house/hakkiri/commit/4979c2aab5de774e24e5684cd9553720733588a0
