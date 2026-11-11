# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. The comments

- [x] 1.1 Probe Vale at `66bbb85` for each comment form in Markdown text and in HTML blocks, and record Vale's alerts in `tests/fixtures/comment-controls/vale.tsv`.  
  Verify the file holds the alerts of every scenario of the requirements, with the Vale commit named in the test.  
  Evidence, 2026-11-11 (UTC): a Vale build of `66bbb85` ran in a box over `tests/fixtures/comment-controls`, nine files and five rules, and gave 304 alerts.  
  The probe found that Vale reads a comment at two moments, which the design records, and that a `raw` rule is gated by `off` alone.
- [x] 1.2 Read the comments of a Markdown file, leave the text between `off` and `on` out of the tree, and gate each block and alert as Vale does.  
  Verify `tests/comment_controls.rs` fails before the change and passes after it.  
  Evidence, 2026-11-11 (UTC): four of the five tests failed on the source before the change, and `every_file_gives_the_alerts_vale_gives` passes after it for the 304 alerts.  
  With the gate of a block, the hiding of an alert, the cut of the text, the end of the element, or the end of a tight item left out, in copies of the tree, it fails.
- [x] 1.3 Read hakkiri's forms, and a level in a comment.  
  Verify `hakkiri_comments_reach_as_vale_comments_do` passes, and the two tests of a level pass.  
  Evidence, 2026-11-11 (UTC): the files of the fixture with each Vale comment written as hakkiri writes it give the same alerts by line, rule and text.  
  A level holds until the rule is named again, and wins over a level of the config.

## 2. The surfaces

- [ ] 2.1 Update `src/instructions.md`, `skills/hakkiri/SKILL.md`, `README.md` and `AGENTS.md` with the hakkiri form first, record the differences in the `vale-differences` spec, and add the patch changeset.  
  Verify `operator_docs` passes, the instructions show `<!-- hakkiri off -->` before Vale's form, and the changeset names `hakkiri: patch`.

## 3. Integration

- [ ] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
