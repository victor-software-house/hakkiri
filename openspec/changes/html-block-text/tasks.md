# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. Probe Vale

- [x] 1.1 Build Vale `66bbb85` in a throwaway Linux box, and run 31 `existence` rules over 31 inputs that hold HTML blocks.  
  Verify `tests/fixtures/html-block/vale.tsv` holds the alerts, and that the probe covers entities, `code`, `tt`, `kbd`, `samp`, `pre`, the text after a raw-text end tag and after `--!>`.
  - 2026-11-10 (UTC), Vale built from source in a throwaway Linux box gave 886 alert rows, kept in [4da5141][c-feat].  
    The inputs cover each case the task names.

## 2. Read the text

- [x] 2.1 Port the tokenizer and Vale's walk to `src/html.rs`, and keep `resets_scope` as a question to that walk.  
  Verify the scope parity test and the tree snapshots pass in CI as before.
  - 2026-11-10 (UTC), `mise run test` passes in the same kind of box, in [4da5141][c-feat].  
    Three tests of `tests/tree.rs` changed, since an HTML block now has text that is prose: the first-place test, which has four quote cases with text, the fenced-block URL test, and the code-and-markup snapshot.  
    CI has not run yet (task 4.1).
- [x] 2.2 Give the text of an HTML block nodes with the scope Vale gives it.  
  Verify the `html-block` snapshot and the `html-block-crlf` snapshot pass in CI.
  - 2026-11-10 (UTC), both snapshots pass in the same box run, in [4da5141][c-feat].
- [x] 2.3 Match Vale on the probe, and list each difference.  
  Verify the `html_parity` test passes in CI, with the rows of `departures.tsv` as its only differences.
  - 2026-11-10 (UTC), the test passes in the same box run.  
    `departures.tsv` holds 25 rows, for six places: an entity, a tag between words, alt text under `text`, a caption, a figure caption and a meta description.

## 3. Record

- [x] 3.1 Update `vale-differences`: remove the known gap, and add each difference with a link to its decision.  
  Verify `openspec validate --all --strict` and the budget check pass in CI.
  - 2026-11-10 (UTC), both pass in a clean checkout, in [9c2a03c][c-docs].

## 4. Integration

- [ ] 4.1 Verify the pull request's CI passes: build, lint, tests, prose lint, OpenSpec and the Template check.

## Workflow follow-up

- Archive this change after the pull request merges, then archive `HKR-021` with `qctl close-from-openspec`.

[c-docs]: https://github.com/victor-software-house/hakkiri/commit/9c2a03ce657f23e2c6c2a04d9378d498f2e42ed1
[c-feat]: https://github.com/victor-software-house/hakkiri/commit/4da5141145c3d3bb30317b2fc3271d859381e116
