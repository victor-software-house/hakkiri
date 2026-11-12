# Tasks

Every compile, lint and test step runs in the pull request's CI, or in a box for a pre-push run, and never on the authoring machine.
This change follows [`gitignore-walk`][walk], which chose the files a run reads.

## 1. The readers

- [x] 1.1 Add the `Text` language and its tree: one `raw` root over the file, with a paragraph for each run of lines between blank lines.  
  Verify the plain-text scenarios of `check-command` pass.
- [x] 1.2 Read `formats` from the config and the packages' settings, and look a file up as Vale's `formatFor` does.  
  Verify the scenarios of `project-config` and `package-settings` for `formats` pass.
- [x] 1.3 Read a selected file with no reader as plain text, honour `on_unknown`, and skip a binary file.  
  Verify the scenarios for `on_unknown` and the binary file pass, and the walk tests that counted a config file still pass.

## 2. The surfaces

- [x] 2.1 Update `src/instructions.md`, `skills/hakkiri/SKILL.md`, `README.md` and `AGENTS.md`, close the plain-text gap in the `vale-differences` spec, and add the patch changeset.  
  Verify `operator_docs` passes, and the changeset names `hakkiri: patch`.

## 3. Integration

- [x] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.  
  Evidence, 2026-11-12 (UTC): PR 107 merged, and CI on main at 930ecee passed: `Build and test`, `Static checks` and `Verify`.

[walk]: ../2026-11-12-gitignore-walk/tasks.md
