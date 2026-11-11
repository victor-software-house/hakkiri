# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The two counts

- [x] 1.1 Probe Vale at `66bbb85` and hakkiri on two sentences with code spans, and pin both counts in tests.  
  Verify `a_code_span_counts_one_word_in_a_metric` and `an_occurrence_counts_no_code_span` pass.  
  Evidence, 2026-11-11 (UTC): Vale built from `66bbb85` in a box gave 3.00 and 3.00 for the metric, 7 and 4 for the mask token, and 3 and 3 for the word token.  
  hakkiri gave 7.00 and 4.00, 3 and 3, and 3 and 3.  
  Both tests pass in a box. They pin counts the engine already gave, so they pass before the change too.
- [x] 1.2 Record the occurrence count in the `vale-differences` spec.  
  Verify the OpenSpec validation passes.  
  Evidence, 2026-11-11 (UTC): `specs`, `specs:budget`, `lint:md`, `lint:typos`, `lint:prose` and `check:repo` pass in a box.

## 2. The rule

- [ ] 2.1 Write `LongSentence` in the prose-style package as a `metric` rule with the `words` formula.  
  Verify its case `counts each code span as a word` passes under `hakkiri test`.

## 3. Integration

- [ ] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
