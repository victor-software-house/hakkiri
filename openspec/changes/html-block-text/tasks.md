# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. Probe Vale

- [x] 1.1 Build Vale `66bbb85` in a throwaway Linux box, and run `existence` rules and `dump` rules over 45 inputs that hold HTML blocks and inline HTML.  
  Verify `tests/fixtures/html-block/vale.tsv` holds the alerts, and that the probe covers references, `code`, `tt`, `kbd`, `samp`, `pre`, joins, captions, descriptions, alt text, scripts and promoted blocks.
  - 2026-11-10 (UTC), Vale built from source in a throwaway Linux box gave 1,659 alert rows, kept in [2673f19][c-feat].  
    A `dump` rule matches the whole text of a block, so a row shows how Vale read it.  
    The probe found that a `quote` rule makes Vale parse the HTML again, so no rule of the probe has that scope.

## 2. Read the text

- [x] 2.1 Port the tokenizer and Vale's walk to `src/html.rs`, with its promotion of block elements inside inline ones and its escaped states for `script`, and keep `resets_scope` as a question to that walk.  
  Verify the scope parity test and the tree snapshots pass in CI as before.
  - 2026-11-10 (UTC), `mise run test` passes in a clean archive of [2673f19][c-feat] in the same kind of box.  
    Three snapshots changed, since the text of a node now carries its joins and masks, and one of them reads `Cell **** text` for a `kbd`.
- [x] 2.2 Give the text of an HTML block nodes with the scope Vale gives it, including captions, `data` and descriptions.  
  Verify the `html-block` snapshot and the `html-block-crlf` snapshot pass in CI.
  - 2026-11-10 (UTC), both snapshots pass in the same box run.
- [x] 2.3 Read character references as Vale does, in HTML and in Markdown text, from a table of `golang.org/x/net/html` v0.62.0.  
  Verify the `references` input matches Vale, apart from the rows of `departures.tsv`.
  - 2026-11-10 (UTC), the `html_parity` test passes in the same box run.
- [x] 2.4 Join the text around inline tags, and mask `kbd` and `samp`, as Vale does, in HTML blocks and in Markdown paragraphs.  
  Verify the `joins-in-blocks`, `joins-in-markdown` and `masks` inputs match Vale, apart from the rows of `departures.tsv`.
  - 2026-11-10 (UTC), the `html_parity` test passes in the same box run.
- [x] 2.5 Match Vale on the probe, and list each difference.  
  Verify the `html_parity` test passes in CI, with the rows of `departures.tsv` as its only differences.
  - 2026-11-10 (UTC), the test passes in the same box run.  
    `departures.tsv` holds 69 rows, for seven places.  
    They are a reference unescaped twice, a match Vale loses beside a reference, a `code` element, and inline HTML elements in a paragraph.  
    The others are a heading of depth 7, a block of masked text only, and a script that never ends.

## 3. Record

- [x] 3.1 Update `vale-differences`: remove the known gap, and add each difference with a link to its decision.  
  Verify `openspec validate --all --strict` and the budget check pass in CI.
  - 2026-11-10 (UTC), both pass in a clean checkout, in [b5adb42][c-docs].

## 4. Integration

- [ ] 4.1 Verify the pull request's CI passes: build, lint, tests, prose lint, OpenSpec and the Template check.

## Workflow follow-up

- Archive this change after the pull request merges, then archive `HKR-021` with `qctl close-from-openspec`.

[c-docs]: https://github.com/victor-software-house/hakkiri/commit/b5adb429527c99b2de9232bd691ad990e0569706
[c-feat]: https://github.com/victor-software-house/hakkiri/commit/2673f190af7adf5e0dd9a874cd4df89e3bc67f79
