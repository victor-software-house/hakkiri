# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The join

- [x] 1.1 Join doc line comments of one type across blank lines in `comments::groups`.  
  Verify the two alert tests for a fence across a blank line and across a line of code, and the `hakkiri test` run of the ProseStyle v0.0.14 `BareURLInComment` rule, fail before the change and pass after it.  
  Evidence, 2026-11-11 (UTC): against the old `comments::groups` in a box, `a_fence_in_doc_comments_stays_open_across_a_blank_line` and `the_cases_of_a_rule_that_reads_a_fence_across_a_blank_line_pass` fail, and the rule's case reports a URL at line 3, column 10.  
  `a_fence_does_not_cross_a_line_of_code_or_a_plain_comment` passes both ways, as a guard. All pass with the change.

## 2. The surfaces

- [x] 2.1 Add the `document-tree` and `vale-differences` deltas and the patch changeset.  
  Verify the OpenSpec validation and `operator_docs` pass.  
  Evidence, 2026-11-11 (UTC): `format`, `lint`, the full suite, `specs`, `specs:budget`, `lint:md`, `lint:typos`, `lint:prose`, `check:versions` and `check:repo` pass in a box.

## 3. Integration

- [ ] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
