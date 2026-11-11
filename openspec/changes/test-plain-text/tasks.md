# Tasks

Every compile, lint and test step runs in the pull request's CI, or in a box for a pre-push run, and never on the authoring machine.

## 1. The reader of a case

- [ ] 1.1 Read a case's input by a `formats` entry, then by the extension, then as plain text.  
  Verify the two scenarios of `test-command` pass, and a test for each fails on the source before the change.

## 2. The surfaces

- [ ] 2.1 Update `src/instructions.md`, `skills/hakkiri/SKILL.md` and `AGENTS.md`, and add the patch changeset.  
  Verify `operator_docs` passes, and the changeset names `hakkiri: patch`.

## 3. Integration

- [ ] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
