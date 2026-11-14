# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The language model

- [x] 1.1 Add `langdex` 0.0.1 to `Cargo.toml` and lock it past the publish-age cooldown with `mise run update:own`.  
  Verify `cargo deny check` passes and `Cargo.lock` holds `langdex 0.0.1`.  
  Evidence, 2026-11-14 (UTC): box run 8x16: `Cargo.lock` holds `langdex 0.0.1`, locked past the cooldown with the publish-age override; the build resolves it. `cargo deny check` is left to the pull request's CI.
- [x] 1.2 Map each reader of `src/language.rs` to and from its langdex entry, and read `from_path`, `extension` and the reader names from langdex; remove `READERS`.  
  Verify the unit tests of `src/language.rs` cover each reader both ways, and `a.mts`, `a.markdown` and `LICENSE`.  
  Evidence, 2026-11-14 (UTC): box run 8x16: the unit tests of `src/language/tests.rs` pass: each reader both ways, the seven extensions, `a.mts`, `a.markdown` and `LICENSE`.

## 2. The language of a file

- [x] 2.1 Give each file the language of its path, then of the interpreter of its first line, and choose its reader by `formats`, its language, then `on_unknown`, in `src/files.rs` and `src/suite.rs`.  
  Verify the scenarios of "A file has a language" and "A reader is chosen by formats, then by extension" pass in `tests/`.  
  Evidence, 2026-11-14 (UTC): box run 8x16: the scenarios of both requirements pass in `tests/check.rs`; the full suite gave 930 passed, 0 failed.
- [x] 2.2 Read a `formats` reader by any langdex name of a language hakkiri reads, in `src/formats.rs`.  
  Verify the scenarios of the `project-config` delta pass.  
  Evidence, 2026-11-14 (UTC): box run 8x16: the `project-config` scenarios pass in `tests/check.rs`, with the new error text.

## 3. Scopes

- [x] 3.1 Parse the last piece of a scope part as a langdex language in `src/scope.rs`, remove `EXTENSIONS`, and match a part against a node's languages in `src/select.rs` and `src/engine.rs`.  
  Verify the scenarios of the `scope-selection` delta and "A file has its own language, and its reader's when formats chose it" pass.  
  Evidence, 2026-11-14 (UTC): box run 8x16: the `scope-selection` scenarios pass in `tests/select.rs`, and `a_mapped_file` in `tests/check.rs`.

## 4. Fences

- [x] 4.1 Give `Node` a language, and parse a fence of a language whose comments hakkiri reads with its grammar in `src/markdown.rs`, mapping each line of its code onto the file.  
  Verify the scenarios of the `document-tree` delta pass, and the tree snapshots and the Markdown corpus change only where a fence holds comments.  
  Evidence, 2026-11-14 (UTC): box run 8x16: the `document-tree` scenarios pass in `tests/alerts.rs` and `tests/tree.rs`; no snapshot and no corpus row changed, since the one fixture fence of a language hakkiri reads holds no comment.
- [x] 4.2 Record each fence comment that the Markdown corpus now reports in its departures, as `vale-differences` sets.  
  Verify `tests/markdown_corpus.rs` passes.  
  Evidence, 2026-11-14 (UTC): box run 8x16: `tests/markdown_corpus.rs` passes with no new row, since the one corpus fence of a language hakkiri reads holds no comment.

## 5. Surfaces

- [x] 5.1 Update `src/instructions.md`, the skill and `AGENTS.md` decision 3, and add the patch changeset.  
  Verify `operator_docs` and `openspec validate --all --strict` pass.  
  Evidence, 2026-11-14 (UTC): box run 8x16: `operator_docs` passes; the changeset was in the plan commit. `openspec validate --all --strict` is left to the pull request's CI.

## 6. Integration

- [ ] 6.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
