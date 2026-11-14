# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The language model

- [ ] 1.1 Add `langdex` 0.0.1 to `Cargo.toml` and lock it past the publish-age cooldown with `mise run update:own`.  
  Verify `cargo deny check` passes and `Cargo.lock` holds `langdex 0.0.1`.
- [ ] 1.2 Map each reader of `src/language.rs` to and from its langdex entry, and read `from_path`, `extension` and the reader names from langdex; remove `READERS`.  
  Verify the unit tests of `src/language.rs` cover each reader both ways, and `a.mts`, `a.markdown` and `LICENSE`.

## 2. The language of a file

- [ ] 2.1 Give each file the language of its path, then of the interpreter of its first line, and choose its reader by `formats`, its language, then `on_unknown`, in `src/files.rs` and `src/suite.rs`.  
  Verify the scenarios of "A file has a language" and "A reader is chosen by formats, then by extension" pass in `tests/`.
- [ ] 2.2 Read a `formats` reader by any langdex name of a language hakkiri reads, in `src/formats.rs`.  
  Verify the scenarios of the `project-config` delta pass.

## 3. Scopes

- [ ] 3.1 Parse the last piece of a scope part as a langdex language in `src/scope.rs`, remove `EXTENSIONS`, and match a part against a node's languages in `src/select.rs` and `src/engine.rs`.  
  Verify the scenarios of the `scope-selection` delta and "A file has its own language, and its reader's when formats chose it" pass.

## 4. Fences

- [ ] 4.1 Give `Node` a language, and parse a fence of a language whose comments hakkiri reads with its grammar in `src/markdown.rs`, mapping each line of its code onto the file.  
  Verify the scenarios of the `document-tree` delta pass, and the tree snapshots and the Markdown corpus change only where a fence holds comments.
- [ ] 4.2 Record each fence comment that the Markdown corpus now reports in its departures, as `vale-differences` sets.  
  Verify `tests/markdown_corpus.rs` passes.

## 5. Surfaces

- [ ] 5.1 Update `src/instructions.md`, the skill and `AGENTS.md` decision 3, and add the patch changeset.  
  Verify `operator_docs` and `openspec validate --all --strict` pass.

## 6. Integration

- [ ] 6.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
