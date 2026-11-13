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

## 3. The setting

- [x] 3.1 Read `code_span_size` from the package's settings file, the project config and a glob, and from `--code-span-size` and its variable, and pass it to the word counts.  
  Verify `tests/code_span_size.rs` and `a_code_span_counts_as_many_words_as_its_size` pass.  
  Evidence, 2026-11-12 (UTC): in a box, `fmt`, `clippy --all-targets -D warnings` and `cargo test --no-fail-fast` pass, 835 tests.
- [ ] 3.2 Probe Vale at `66bbb85` and hakkiri with the setting at 0 and at its default on the same files, for the `metric` and `readability` rules over each scope that holds a code span.  
  Verify at 0 the alerts equal Vale's, and by default they equal the alerts before the change.  
  Evidence, 2026-11-12 (UTC), Vale built from `66bbb85` in a box, 16 `metric` and `readability` rules over the scopes sentence, paragraph, list, table cell, heading and file.  
  By default hakkiri gives the alerts of the build before the change, 0 differing of 28,645 over 5 hand-written files and 211 Markdown files.  
  With `--code-span-size 0` it gives Vale's alerts on 8 hand-written files, apart from 4 that Vale lacks.  
  Three are table header cells, which hakkiri measures with or without a code span, and one is a sentence that starts with a code span in the middle of a paragraph.  
  On the 211 files the alerts that differ from Vale's fall from 18,424 by default to 1,910 at 0, 584 of them table cells.  
  The rest are not attributed yet, so the task stays open.
- [x] 3.3 Update the instructions, the skill and `AGENTS.md` with the key, the flag, the variable and the unit, words.

## 4. Integration

- [ ] 4.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
