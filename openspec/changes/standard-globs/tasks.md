# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The glob

- [ ] 1.1 Compile every `files` and `formats` glob through `config::standard_glob`, with a separator that `*` does not cross, `**/` before a glob with no `/`, a leading `/` dropped, and `**` after a trailing `/`.  
  Verify the unit tests of `src/config/tests.rs` for each rule pass.
- [ ] 1.2 Update `src/instructions.md` and `AGENTS.md` decision 5, and add the patch changeset.  
  Verify `operator_docs` and `openspec validate --all --strict` pass.

## 2. Integration

- [ ] 2.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
