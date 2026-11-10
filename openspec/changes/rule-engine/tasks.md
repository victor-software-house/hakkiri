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
  - 2026-11-10 (UTC), the two-fences-and-an-indented-block test passes in the same box run, in [462eaa9][c-tree].

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
    An item of only `~` parts reads as `text`.  
    A last part from Vale's extension list narrows the scope, so a rule for another language selects nothing.
- [x] 4.2 Build each node's check text with its byte table, the code span word and blanked inline exclusions.  
  Verify the inline markup, code span and inline exclusion scenarios pass in CI.
  - 2026-11-10 (UTC), the markup, code span and exclusion tests pass in the same box run, in [3fdac6d][c-select].  
    A comment node reads as the Markdown inside it, without its markers.
- [x] 4.3 Cut sentences with the `HKR-015` splitter and words with prose's word pattern.  
  Verify the sentence scenario and the word count scenarios of the rule-files spec pass in CI.
  - 2026-11-10 (UTC), the sentence test and the two word count tests pass in the same box run, in [3fdac6d][c-select].  
    The counts are of `words` over the selected text; the `occurrence` rule that reports them comes with task 6.3.  
    `words` reads alphabetic characters as letters, which adds a few combining marks to prose's `\p{L}`.

## 5. Patterns and alerts

- [ ] 5.1 Compile each pattern with `regex`, falling back to `fancy-regex` with its backtrack limit, and apply Vale's word, case and class rules.  
  Verify the backtrack scenario gives one error alert and no panic in CI.
- [ ] 5.2 Place alerts by the line table, fill messages, sort, drop duplicates and apply `limit`.  
  Verify every [alerts scenario][alerts] passes as a unit test in CI.
- [ ] 5.3 Run every check type of this change over a 50,000-character line and a 60,000-line file.  
  Verify both long-input tests pass in CI with no panic, and record their run time in the evidence note.

## 6. Checks

- [ ] 6.1 Build `existence`, with `raw`, `append`, `nonword`, `ignorecase` and `exceptions`.  
  Verify its [match-checks scenarios][match] and its rule-files scenarios pass as unit tests in CI.
- [ ] 6.2 Build `substitution`, with longer keys first, the already-correct skip, `capitalize` and `$1` values.  
  Verify its match-checks and rule-files scenarios pass as unit tests in CI.
- [ ] 6.3 Build `occurrence`, with `max`, `min` and the alert place.  
  Verify its match-checks and rule-files scenarios pass as unit tests in CI.
- [ ] 6.4 Build `metric`: the variables, `occurrences`, element counts, the evaluator, the condition with `result`, and load errors.  
  Verify every [metric-check scenario][metric] and the metric scenarios of the rule-files spec pass as unit tests in CI.

## 7. Flesch-Kincaid parity

- [x] 7.1 Measure Vale 3.26.0's `flesch_kincaid`, `words`, `sentences` and `syllables` for each prose-style case input with no code span.  
  Run it once in a throwaway Linux box, with a `metric` rule whose message prints each value.  
  Verify the measurement covers every such input.
- [x] 7.2 Store the values in a fixture file under `tests/fixtures/` that names the Vale version and the prose-style commit.  
  Verify the fixture holds one entry per input.
- [ ] 7.3 Compare hakkiri's values with the fixture at the tolerance the design states.  
  Verify the parity test passes in CI and prints each input whose counts differ from Vale's.

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

- [ ] 8.1 Verify the pull request's CI passes: build, lint, tests, docs, deny, prose lint, OpenSpec and the Template check.

## Workflow follow-up

- Archive this change after the pull request merges, then archive `HKR-004` with `qctl close-from-openspec`.

[design]: design.md
[c-deny]: https://github.com/victor-software-house/hakkiri/commit/f23a9c9776a9e45e2acac151b825c0fdeedb2fb8
[c-tree]: https://github.com/victor-software-house/hakkiri/commit/462eaa9e82ec229444a658904a23fbcd5be274ac
[c-select]: https://github.com/victor-software-house/hakkiri/commit/3fdac6d3dcd3d3929577c34ff37f3e48a222e050
[c-load]: https://github.com/victor-software-house/hakkiri/commit/389d5b4791be8248e954e303fd78f9314d1111ca
[segments]: specs/node-segments/spec.md
[tree]: specs/document-tree/spec.md
[loading]: specs/rule-loading/spec.md
[selection]: specs/scope-selection/spec.md
[alerts]: specs/alerts/spec.md
[match]: specs/match-checks/spec.md
[metric]: specs/metric-check/spec.md
[rule-files]: ../../specs/rule-files/spec.md
[ps-commit]: https://github.com/victor-software-house/prose-style/tree/ee9f207279864e305d7372aef4a6c5dacc3eb846
[fk-tsv]: ../../../tests/fixtures/fk-parity/vale-fk.tsv
[fk-inputs]: ../../../tests/fixtures/fk-parity/inputs
[fk-sources]: ../../../tests/fixtures/fk-parity/SOURCES.md
