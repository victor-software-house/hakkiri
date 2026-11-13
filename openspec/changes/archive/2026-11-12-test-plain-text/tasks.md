# Tasks

Every compile, lint and test step runs in the pull request's CI, or in a box for a pre-push run, and never on the authoring machine.

## 1. The reader of a case

- [x] 1.1 Read a case's input by a `formats` entry, then by the extension, then as plain text.  
  Verify the two scenarios of `test-command` pass, and a test for each fails on the source before the change.  
  Evidence, 2026-11-12 (UTC): PR 109 added the two tests, and both failed on the source before the change and pass after it.  
  PR 112 added three more, and two of them failed before the change.

## 2. The surfaces

- [x] 2.1 Update `src/instructions.md`, `skills/hakkiri/SKILL.md` and `AGENTS.md`, and add the patch changeset.  
  Verify `operator_docs` passes, and the changeset names `hakkiri: patch`.  
  Evidence, 2026-11-12 (UTC): PRs 109 and 112 updated `src/instructions.md` and added `.changeset/test-plain-text.md`, a patch, and `operator_docs` passes in the suite of main.

## 3. Integration

- [x] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.  
  Evidence, 2026-11-12 (UTC): PRs 109 and 112 merged, and CI on main at 930ecee passed: `Build and test`, `Static checks` and `Verify`.
