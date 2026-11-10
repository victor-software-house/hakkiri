# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. Probe Vale

- [ ] 1.1 Build Vale `66bbb85` in a throwaway Linux box, and run 31 `existence` rules over 31 inputs that hold HTML blocks.  
  Verify `tests/fixtures/html-block/vale.tsv` holds the alerts, and that the probe covers entities, `code`, `tt`, `kbd`, `samp`, `pre`, the text after a raw-text end tag and after `--!>`.

## 2. Read the text

- [ ] 2.1 Port the tokenizer and Vale's walk to `src/html.rs`, and keep `resets_scope` as a question to that walk.  
  Verify the scope parity test and the tree snapshots pass in CI as before.
- [ ] 2.2 Give the text of an HTML block nodes with the scope Vale gives it.  
  Verify the `html-block` snapshot and the `html-block-crlf` snapshot pass in CI.
- [ ] 2.3 Match Vale on the probe, and list each difference.  
  Verify the `html_parity` test passes in CI, with the rows of `departures.tsv` as its only differences.

## 3. Record

- [ ] 3.1 Update `vale-differences`: remove the known gap, and add each difference with a link to its decision.  
  Verify `openspec validate --all --strict` and the budget check pass in CI.

## 4. Integration

- [ ] 4.1 Verify the pull request's CI passes: build, lint, tests, prose lint, OpenSpec and the Template check.

## Workflow follow-up

- Archive this change after the pull request merges, then archive `HKR-021` with `qctl close-from-openspec`.
