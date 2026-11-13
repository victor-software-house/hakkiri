# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The splitter

- [x] 1.1 Pass the code ranges to the splitter, and split around a code span and after a unit of a number.  
  Verify the code span and unit cases of `src/splitter/tests.rs`.
- [x] 1.2 End no sentence at `vs.`, and read no date as a list number.  
  Verify `versus_ends_no_sentence` and `a_date_is_no_list_number` in `src/splitter/tests.rs`.

## 2. The differences from Vale

- [x] 2.1 Record each place where Vale splits otherwise in `tests/fixtures/sentence-differences`, with Vale's alerts at 66bbb85 in `vale.tsv`.  
  Verify `vale_and_hakkiri_split_these_sentences_otherwise` in `tests/sentence_differences.rs`.
- [x] 2.2 Update `src/instructions.md`, the skill and `AGENTS.md`, and add the changeset.  
  Verify `operator_docs` passes.

## 3. Integration

- [ ] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
- [ ] 3.2 Run the ProseStyle alert diff of `HKR-008` again, and verify LongSentence and SentenceLength are the same as Vale or better.

[ledger]: ../../../tasks.yaml
