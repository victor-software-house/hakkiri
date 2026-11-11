# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The fill rule

- [x] 1.1 Probe Vale at `66bbb85` for messages with indexed verbs, and record the alerts in `tests/fixtures/sequence-verbs/vale.tsv`.  
  Verify the file holds the alerts of ProseStyle v0.0.14 `NounStack` and `ThingsSpeakNoun` and of the three rules that take values by number.  
  Evidence, 2026-11-11 (UTC): a Vale build of `66bbb85` ran over the new inputs and over the whole `sequence` fixture. It reproduced all 446 rows of `tests/fixtures/sequence/vale.tsv`, and gave 11 rows for the new rules.
- [x] 1.2 Fill `%[n]s` in `fill`, with a plain verb after it taking the next value.  
  Verify `fills_indexed_verbs_as_vale_does` and the scenarios of the `alerts` delta fail before the change and pass after it.  
  Evidence, 2026-11-11 (UTC): against the old `fill` in a box, `fills_indexed_verbs_as_vale_does`, `values_taken_by_number`, `a_plain_verb_after_an_indexed_one`, `a_value_used_twice` and `a_verb_with_no_value` fail. `a_bracket_that_is_no_verb_stays_as_written` passes both ways, as a guard. All pass with the change.
- [x] 1.3 Run the ProseStyle v0.0.14 cases of the two rules through `hakkiri test`.  
  Verify `tests/test_command.rs` passes with both files, and each case reports its message.  
  Evidence, 2026-11-11 (UTC): `the_cases_of_two_rules_with_indexed_verbs_pass` runs the two rule files as ProseStyle v0.0.14 ships them, and reports `2 files, 6 passed, 0 failed`. It fails with the old `fill`.

## 2. The surfaces

- [x] 2.1 Check `src/instructions.md`, the skill, `README.md` and `AGENTS.md` for a statement of the fill rule, add the `vale-differences` scenario and the patch changeset.  
  Verify `operator_docs` passes and the changeset names `hakkiri: patch`.  
  Evidence, 2026-11-11 (UTC): none of those four files states the fill rule, so none changes. The full suite, `lint`, `doc`, `specs`, `lint:md` and `lint:prose` pass in a box.

## 3. Integration

- [ ] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
