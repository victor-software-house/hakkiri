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
- [x] 1.4 Read the `{/* … */}` comments of an MDX file as the HTML comments of a Markdown file.  
  Verify the parity test passes over the MDX file of the fixture, in Vale's form and in hakkiri's.  
  Evidence, 2026-11-11 (UTC): Vale 66bbb85 gave 46 alerts over `c10.mdx`, and the 350 rows of the fixture match.
- [x] 1.5 Probe the cases of the first probe that it left open, and fix each one where hakkiri read less than Vale.  
  Verify `every_file_gives_the_alerts_vale_gives` passes over the 23 files and 1,238 rows of the `text` folder, and `vale_places_a_comment_wrongly_where_hakkiri_places_it_right` passes.  
  Evidence, 2026-11-12 (UTC): Vale `66bbb85` and hakkiri ran over setext and ATX headings, links, images, references, nested quotes and lists, task lists and inline HTML.  
  They also ran over `details`, `pre`, nested `div`, fenced and indented code, tables in lists, odd comment spellings, non-ASCII text, CRLF lines and footnotes.  
  The two differed in six ways, apart from footnotes.  
  Four were fixed.  
  The first is the text of a tight item before a block that opens in it.  
  The second is the text of an HTML block at the tag that flushes it.  
  The third is the text left at the end of an HTML block.  
  The fourth is the text of a paragraph before a `div` tag inside it.  
  Two are Vale's flaws, recorded in the `vale-differences` spec.  
  Footnote definitions differ for reasons that are not comments, and move to row `HKR-027`.

## 2. The surfaces

- [x] 2.1 Update `src/instructions.md`, `skills/hakkiri/SKILL.md`, `README.md` and `AGENTS.md` with the hakkiri form first, record the differences in the `vale-differences` spec, and add the patch changeset.  
  Verify `operator_docs` passes, the instructions show `<!-- hakkiri off -->` before Vale's form, and the changeset names `hakkiri: patch`.  
  Evidence, 2026-11-11 (UTC): the full suite, 645 tests, passed in a box with `cargo fmt`, `clippy -D warnings` on the host and musl targets, `cargo deny` and `cargo machete`, and `operator_docs` with it.  
  The Mac gate passed: the prose lint, rumdl, typos, workflow lint, workflow policy, `check:local`, `specs:budget`, `format:toml`, `qctl check` and `openspec validate --all --strict`.

## 3. Integration

- [x] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.  
  Evidence, 2026-11-12 (UTC): PRs 98 and 103 passed CI before they merged, and main passed after.
