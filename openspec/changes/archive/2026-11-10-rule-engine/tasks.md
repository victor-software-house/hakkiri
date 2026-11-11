# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. Open decisions

- [x] 1.1 Record the operator's answer to each of the seven open decisions in [design.md][design], moving each into Decisions with the candidates that lost.  
  Verify the Open decisions section is empty and `mise run specs:budget` passes.  
  2026-11-10, the operator's answers are recorded as decisions, and the Open decisions section is gone.
- [x] 1.2 Add `regex` 1.13.1, `fancy-regex` 0.20.1 and `ctl-core` 0.7.0 with its `input` feature to `Cargo.toml`.  
  Verify the `deny` and `deps` steps pass in CI.
  - 2026-11-10 (UTC), `cargo deny check licenses bans sources` and `cargo-machete --with-metadata` pass in a throwaway Linux box, in [f23a9c9][c-deny].  
    `encoding_rs` needed `BSD-3-Clause` in the template answers and `deny.toml`.  
    The rule files also need `serde` and `garde` as direct dependencies, for the derives.  
    CI has not run yet (task 8.1).

## 2. The tree

- [x] 2.1 Make inline markup and URLs gaps between segments, so a link, a code span and an image read as their text.  
  Review the changed tree snapshots.  
  Verify every [node-segments scenario][segments] passes as a test in CI.
  - 2026-11-10 (UTC), the four scenario tests and 77 tree tests pass in the same box run, in [462eaa9][c-tree].  
    Nine snapshots changed and were read.  
    A strikethrough loses its tildes too.  
    A paragraph that holds only a URL has no text, so its link node moves up a level.
- [x] 2.2 Count the fenced and indented code blocks of a Markdown or MDX tree.  
  Verify the [document-tree scenario][tree] passes as a test in CI.
  - 2026-11-10 (UTC), the test of two fences and an indented block passes in the same box run, in [462eaa9][c-tree].

## 3. Loading

- [x] 3.1 Decode each rule file into the rule type of its check.  
  Refuse a missing, unknown or wrongly typed field with its file and line.  
  Verify every [rule-loading scenario][loading] passes as a unit test in CI.
  - 2026-11-10 (UTC), the 24 loading tests pass in the same box run, in [389d5b4][c-load].  
    The YAML parser reports a tab in an indentation on the line of the scalar above it, so loading moves the error to the line of the tab.
- [x] 3.2 Refuse `affix`, `doc(...)` scopes and `script` with an error that names them, and load `vocab` and `action` with no effect.  
  Verify a unit test per field passes in CI.
  - 2026-11-10 (UTC), one test per field passes in the same box run, in [389d5b4][c-load].  
    The seven check types of later changes are refused by name too.  
    `matchcase` loads with no effect, as `action` does.

## 4. Selection and text

- [x] 4.1 Parse scopes with `&`, `~`, lists and extension parts, and match parts by dotted prefix, with `text`, `paragraph` and `sentence` as the design states.  
  Verify every [scope-selection scenario][selection] and every scope scenario in the [rule-files spec][rule-files] pass as unit tests in CI.
  - 2026-11-10 (UTC), 23 selection tests and the unknown-scope load test pass in a Linux box, in [3fdac6d][c-select].  
    They assert the selected blocks and their file bytes, since alerts come with task 5.2.  
    An item of only `~` parts reads as `text`, unless every part names another extension.  
    A last part from Vale's extension list narrows the scope.
  - 2026-11-10 (UTC), the `~heading.rs` test passes in a Linux box run of [9220201][c-scope-only]: an item whose every part names another extension selects nothing.
- [x] 4.2 Build each node's check text with its byte table, the code span word and blanked inline exclusions.  
  Verify the inline markup, code span and inline exclusion scenarios pass in CI.
  - 2026-11-10 (UTC), the markup, code span and exclusion tests pass in the same box run, in [3fdac6d][c-select].  
    A comment node reads as the Markdown inside it, without its markers.
- [x] 4.3 Cut sentences with the `HKR-015` splitter and words with prose's word pattern.  
  Verify the sentence scenario and the word count scenarios of the rule-files spec pass in CI.
  - 2026-11-10 (UTC), the sentence test and the two word count tests pass in the same box run, in [3fdac6d][c-select].  
    The counts are of `words` over the selected text; the `occurrence` rule that reports them comes with task 6.3.  
    `words` runs prose's own pattern, so a combining mark ends a word.
  - 2026-11-10 (UTC), the combining mark test passes in a box run of [e5c502e][c-words].

## 5. Patterns and alerts

- [x] 5.1 Compile each pattern with `regex`, falling back to `fancy-regex` with its backtrack limit, and apply Vale's word, case and class rules.  
  Verify the backtrack scenario gives one error alert and no panic in CI.
  - 2026-11-10 (UTC), the backtrack test passes in a Linux box, in [0647690][c-alerts].  
    `Team.Slow` gives one `error` alert that names the limit, and the other rule reports.  
    fancy-regex finishes the spec's `(a+)+(?=b)` at once, so the test and the scenario use `(a+)+(?=b)c`.
- [x] 5.2 Place alerts by the line table, fill messages, sort, drop duplicates and apply `limit`.  
  Verify every [alerts scenario][alerts] passes as a unit test in CI.
  - 2026-11-10 (UTC), a test for each alerts scenario passes in the same box run, in [0647690][c-alerts].  
    The percent scenario gained `ignorecase: true`, which its capital letter needs.  
  - 2026-11-10 (UTC), the markup test and the wide character test pass in a box run of [6d38974][c-last-column].  
    The end column goes through the byte table, as the start does.  
    A zero-width match keeps both columns equal, in [28e2bd4][c-zero].  
    No Vale binary was at hand, so only its source was read, not run.
- [x] 5.3 Run every check type of this change over a 50,000-character line and a 60,000-line file.  
  Verify both long-input tests pass in CI with no panic, and record their run time in the evidence note.
  - 2026-11-10 (UTC), both tests pass for all four check types in a Linux box, in a debug build: 88 ms for 50,000 characters, 27.7 s for 60,000 lines, in [d324f02][c-metric].  
    `parse` takes nearly all of it: markdown-rs grows with the square of the line count, 2.5 s alone in a release build.

## 6. Checks

- [x] 6.1 Build `existence`, with `raw`, `append`, `nonword`, `ignorecase` and `exceptions`.  
  Verify its [match-checks scenarios][match] and its rule-files scenarios pass as unit tests in CI.
  - 2026-11-10 (UTC), the five `existence` scenarios of the match-checks spec pass in the same box run, in [0647690][c-alerts].  
    The rule-files spec has no `existence` scenario.  
    The alerts tests need a check to run, so this task moved up.  
  - 2026-11-10 (UTC), the nested phrases test passes in a box run of [f59a77f][c-phrases], and the 50,000-character test runs in 95 ms with a phrase that matches throughout.  
    A scratch case of 25,000 matches in 25,000 phrase spans took 0.6 s, since `excepted` scanned every span per match.  
  - 2026-11-10 (UTC), `excepted` looks a match up by binary search in [7cbdcc5][c-linear]: 50,000 matches in 50,000 spans take 118 ms, and the test holds them to 5 s.  
    The 50,000-character test takes 92 ms.  
    Tests cover a phrase in another case and two phrase searches that give up, as one error alert.
- [x] 6.2 Build `substitution`, with longer keys first, the already-correct skip, `capitalize` and `$1` values.  
  Verify its match-checks and rule-files scenarios pass as unit tests in CI.
  - 2026-11-10 (UTC), the five `substitution` scenarios of the match-checks spec pass in a Linux box, in [3868424][c-checks].  
    The rule-files spec has no `substitution` scenario.  
    A sixth test shows `1st` is not capitalized, a difference from Vale in design decision 16.  
  - 2026-11-10 (UTC), the key across a line wrap passes in a box run of [91fc2e4][c-expander].
- [x] 6.3 Build `occurrence`, with `max`, `min` and the alert place.  
  Verify its match-checks and rule-files scenarios pass as unit tests in CI.
  - 2026-11-10 (UTC), the two `occurrence` scenarios of the match-checks spec pass in the same box run, in [3868424][c-checks].  
    The rule-files spec has no `occurrence` scenario.  
  - 2026-11-10 (UTC), three tests pass in a box run of [1341734][c-occ-code]: a match in code is not counted, one outside is, and none in code misses a minimum.  
  - 2026-11-10 (UTC), a token that matches only blanks points at the first word under `min`, in [eb734af][c-blank].
- [x] 6.4 Build `metric`: the variables, `occurrences`, element counts, the evaluator, the condition with `result`, and load errors.  
  Verify every [metric-check scenario][metric] and the metric scenarios of the rule-files spec pass as unit tests in CI.
  - 2026-11-10 (UTC), 13 metric tests cover every metric-check scenario and both rule-files scenarios, with 7 formula tests, in a Linux box, in [d324f02][c-metric].  
    All 4,383 words of prose's syllable data match.  
    Two of its three score cases match to 0.01, and the third has two sentences here, where prose finds one.  
  - 2026-11-10 (UTC), the Rust comment test and the heading condition test pass in a box run of [4fabe24][c-condition].  
    A whole-file metric now measures the comments of a code file, from [f312d38][c-comments], which Vale's default does not do ([decision 16][design-16]).  
    Eight such rules over a 50,000-character line take 42 ms, from 327 ms, in [7950466][c-once].  
  - 2026-11-10 (UTC), after one warm-up run, the median of five runs on a 4x8 box is 42 ms for one rule and 42 ms for eight.  
    Before [7950466][c-once] it was 43 ms and 341 ms.  
    The earlier 66 ms for one rule was a single run with no warm-up.
  - 2026-11-10 (UTC), a test lints 60,000 lines with one metric rule and with eight, in [df67f66][c-long-metric].  
    Eight take 463 ms and one takes 488 ms, and the test holds eight to under three times one.  
    Before [7950466][c-once] eight took 3.59 s against 480 ms, which fails that bound.

## 7. Flesch-Kincaid parity

- [x] 7.1 Measure Vale 3.26.0's `flesch_kincaid`, `words`, `sentences` and `syllables` for each prose-style case input with no code span.  
  Run it once in a throwaway Linux box, with a `metric` rule whose message prints each value.  
  Verify the measurement covers every such input.
- [x] 7.2 Store the values in a fixture file under `tests/fixtures/` that names the Vale version and the prose-style commit.  
  Verify the fixture holds one entry per input.
- [x] 7.3 Compare hakkiri's values with the fixture at the tolerance the design states.  
  Verify the parity test passes in CI and prints each input whose counts differ from Vale's.
  - 2026-11-10 (UTC), the parity test passes in the same box run: 51 inputs, counts equal Vale's on all 51, scores within 0.01, in [d324f02][c-metric].
  - 2026-11-10 (UTC), the test also covers the 10 Rust inputs where Vale prints no value, in [ff43618][c-rust-parity].  
    Six hold a comment, and hakkiri counts 18, 21, 9, 3, 2 and 4 words in them.  
    The other four hold only strings, and hakkiri gives nothing, as Vale does.

### Evidence for 7.1

- 2026-11-10 UTC: Vale 3.26.0, the official Linux release, ran once in a throwaway Linux box with four `metric` rules, one per value.
- The inputs are the 87 cases of [prose-style at `ee9f207`][ps-commit].
  21 hold a code span or fence and are left out, which leaves 66.
- Vale printed values for 51 of the 66.
  It printed none for 15: 10 Rust inputs, 4 Markdown tables and 1 line that holds only a URL.
  The fixture marks them `no-words`.
- The rule messages print `%.2f`, so the score was multiplied by 1,000,000 in the formula to keep six decimals.
- Not verified: Vale's scores for Rust inputs, which its summary leaves empty.

### Evidence for 7.2

- 2026-11-10 UTC: [`vale-fk.tsv`][fk-tsv] holds one row per input, 66 rows, each with the Vale version and the prose-style commit.
- The 15 `no-words` rows stay in the fixture with that mark, so task 7.3 compares the other 51.
- The inputs are copied under [`inputs/`][fk-inputs], from a repository the same owner holds.
- How to rerun the measurement, the format and the license note are in [`SOURCES.md`][fk-sources].

## 8. Integration

- [x] 8.1 Verify the pull request's CI passes: build, lint, tests, docs, deny, prose lint, OpenSpec and the Template check.
  - 2026-11-10 (UTC), CI passes on [08a2d35][c-horizon] of [PR 23][pr-23]: Build and test, Static checks, Verify, CodeQL, both Analyze jobs, secrets and the Kody review.  
    The Template check and Publish checks skip on this branch, and I did not run them.

## Workflow follow-up

- Archive this change after the pull request merges, then archive `HKR-004` with `qctl close-from-openspec`.

[design]: design.md
[design-16]: design.md#16-better-than-vale-where-vale-is-wrong
[c-deny]: https://github.com/victor-software-house/hakkiri/commit/f23a9c9776a9e45e2acac151b825c0fdeedb2fb8
[c-tree]: https://github.com/victor-software-house/hakkiri/commit/462eaa9e82ec229444a658904a23fbcd5be274ac
[c-select]: https://github.com/victor-software-house/hakkiri/commit/3fdac6d3dcd3d3929577c34ff37f3e48a222e050
[c-metric]: https://github.com/victor-software-house/hakkiri/commit/d324f02867451066e0af61e255c22fd803e36d5f
[c-checks]: https://github.com/victor-software-house/hakkiri/commit/386842412facac10075a4b35a9ea49f0961a21e8
[c-alerts]: https://github.com/victor-software-house/hakkiri/commit/06476907b5853a47114fceac438d3ce5125c86f3
[c-words]: https://github.com/victor-software-house/hakkiri/commit/e5c502ed40a89137e68af0149b8c1b59d2ccef29
[c-scope-only]: https://github.com/victor-software-house/hakkiri/commit/92202016fad285a8d5771be70607f854c039451a
[c-phrases]: https://github.com/victor-software-house/hakkiri/commit/f59a77f3b81f8dc41b84e3ec6bb3db0b598ae91c
[c-last-column]: https://github.com/victor-software-house/hakkiri/commit/6d389747e25ba07db83d15c90587a0b9d9362d0f
[c-occ-code]: https://github.com/victor-software-house/hakkiri/commit/1341734eede62760dbfb78284a10a8fa60265dc3
[c-expander]: https://github.com/victor-software-house/hakkiri/commit/91fc2e4c6dbac9f093d47ead2bd433654388fb53
[c-comments]: https://github.com/victor-software-house/hakkiri/commit/f312d389cc2566bdaf3ced6fbc153f4c3173a456
[c-once]: https://github.com/victor-software-house/hakkiri/commit/7950466010eebccf5c05016a6a2d784e76c8671f
[c-condition]: https://github.com/victor-software-house/hakkiri/commit/4fabe241fbc5a9b973a0cc12e699f92dac9e321d
[c-linear]: https://github.com/victor-software-house/hakkiri/commit/7cbdcc598fd42e2940db9746f9ebaee574b25722
[c-horizon]: https://github.com/victor-software-house/hakkiri/commit/08a2d35e74598223592a5f4547f38b64f7952157
[pr-23]: https://github.com/victor-software-house/hakkiri/pull/23
[c-long-metric]: https://github.com/victor-software-house/hakkiri/commit/df67f660e42145161b46280a4a3c321b899663cc
[c-rust-parity]: https://github.com/victor-software-house/hakkiri/commit/ff436183465279d09d9621cf8883bd2401fbbce9
[c-zero]: https://github.com/victor-software-house/hakkiri/commit/28e2bd40b28d871c0e5802e7ac14a29f21c84c9d
[c-blank]: https://github.com/victor-software-house/hakkiri/commit/eb734afda9d33ab272ab83d0bde22e0fbf8dab6b
[c-load]: https://github.com/victor-software-house/hakkiri/commit/389d5b4791be8248e954e303fd78f9314d1111ca
[segments]: specs/node-segments/spec.md
[tree]: specs/document-tree/spec.md
[loading]: specs/rule-loading/spec.md
[selection]: specs/scope-selection/spec.md
[alerts]: specs/alerts/spec.md
[match]: specs/match-checks/spec.md
[metric]: specs/metric-check/spec.md
[rule-files]: ../../../specs/rule-files/spec.md
[ps-commit]: https://github.com/victor-software-house/prose-style/tree/ee9f207279864e305d7372aef4a6c5dacc3eb846
[fk-tsv]: ../../../../tests/fixtures/fk-parity/vale-fk.tsv
[fk-inputs]: ../../../../tests/fixtures/fk-parity/inputs
[fk-sources]: ../../../../tests/fixtures/fk-parity/SOURCES.md
