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

- [x] 2.1 Write `LongSentence` in the prose-style package as a `metric` rule with the `words` formula.  
  Verify its case `counts each code span as a word` passes under `hakkiri test`.  
  Evidence, 2026-11-14 (UTC): ProseStyle v0.0.16 ships `LongSentence` as a `metric` rule with `formula: words`, and `hakkiri test` 0.0.6 passes its 3 cases, `counts each code span as a word` among them.

## 3. The setting

- [x] 3.1 Read `code_span_size` from the package's settings file, the project config and a glob, and from `--code-span-size` and its variable, and pass it to the word counts.  
  Verify `tests/code_span_size.rs` and `a_code_span_counts_as_many_words_as_its_size` pass.  
  Evidence, 2026-11-12 (UTC): in a box, `fmt`, `clippy --all-targets -D warnings` and `cargo test --no-fail-fast` pass, 835 tests.
- [x] 3.2 Probe Vale at `66bbb85` and hakkiri with the setting at 0 and at its default on the same files, for the `metric` and `readability` rules over each scope that holds a code span.  
  Give a cause to each difference.  
  Verify at 0 the alerts equal Vale's apart from the differences that `vale-differences` records and that rows [HKR-064][ledger] and [HKR-065][ledger] take, and by default they equal the alerts before the change.  
  Evidence, 2026-11-12 (UTC), Vale built from `66bbb85` in a box, 16 `metric` and `readability` rules over the scopes sentence, paragraph, list, table cell, heading and file.  
  By default hakkiri gives the alerts of the build before the change, 0 differing of 28,645 over 5 hand-written files and 211 Markdown files.  
  With `--code-span-size 0` the alerts that differ from Vale's fall from 18,424 by default to 1,910, 584 of them table cells.  
  A build of Vale with its duplicate-position drop off differs in 860 of them, so the drop is the cause of 1,050: table cells 584 to 0, headings 19 to 0, list items 71 to 2.  
  Of the 860, 70 are the text of a link that repeats its address, which [HKR-064][ledger] fixes, and 790 are in how sentences split.  
  172 minimal inputs for the sentence differences fall into four groups: a full stop after a code span, a full stop after a number, a number that starts a heading or a task item, and a few others.  
  Each input still differs from Vale on its own, and is handed to [HKR-065][ledger].  
  `design.md` decisions 6 to 8 give the causes, and the `vale-differences` delta records the drop of an alert.
- [x] 3.3 Update the instructions, the skill and `AGENTS.md` with the key, the flag, the variable and the unit, words.
- [ ] 3.4 Close or record, in [HKR-065][ledger], the 790 differences in how sentences split.  
  Verify the probe of 3.2 at 0, with Vale's drop off, shows no difference that no row records.

## 4. Integration

- [ ] 4.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.

[ledger]: ../../../tasks.yaml
