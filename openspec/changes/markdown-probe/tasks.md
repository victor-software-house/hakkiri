# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The probe

- [x] 1.1 Probe Vale at `66bbb85` and hakkiri over 111 inputs of one construct each, and record the rows in `tests/fixtures/markdown-corpus`.  
  Verify `tests/markdown_corpus.rs` fails on any difference that `departures.tsv` does not list.  
  Evidence, 2026-11-12 (UTC): in a cloud box, Vale gave 2155 rows over the inputs, and before this change hakkiri differed on tables, escapes, footnotes, links and inline HTML.  
  The first probe ran natural prose and showed rows from Vale's search for a repeated word, so each input then used words that no other line of its file uses.

## 2. The fixes

- [ ] 2.1 Read tables, escapes, links, `ftp://` addresses, footnotes and inline HTML elements as Vale does.  
  Verify `tests/markdown_corpus.rs`, `tests/tree.rs` and the whole `cargo test` pass.

## 3. The surfaces

- [ ] 3.1 Add the deltas of the `node-segments` and `vale-differences` specs, and the patch changeset.  
  Verify `openspec validate --all --strict` passes.

## 4. Integration

- [ ] 4.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
