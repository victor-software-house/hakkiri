# Tasks

Every compile, lint and test step runs in the pull request's CI, or in a box for a pre-push run.  
None runs on the authoring machine.

## 1. The walk and the options

- [x] 1.1 Walk with the `ignore` crate, and skip what git ignores unless `gitignore` is false.  
  Keep the skips for `node_modules`, `.git`, the package folders and folders off the way to the root.  
  Verify the walk scenarios of `check-command` pass, and the existing walk tests still pass.

  Evidence, 2026-11-11 (UTC): the new walk tests failed on the source before the change.  
  They pass after it in a box, with the 66 tests of `check`.
- [x] 1.2 Warn about a given file that no glob selects, with `on_unmatched` for `warn`, `error` and `ignore`.  
  Verify the unmatched scenarios of `check-command` pass, including the file given twice and the file whose styles contain no rule.  
  Evidence, 2026-11-11 (UTC): the five tests of the scenarios failed before the change and pass after it.
- [x] 1.3 Read `fail_level`, `gitignore` and `on_unmatched` from the config, below the flag and the variable, and reject them in a package's settings.  
  Verify the scenarios of `project-config` and `package-settings` pass.

  Evidence, 2026-11-11 (UTC): three groups of tests failed before the change and pass after it.  
  They are the order tests for the three keys, the two rejected values and the package test.

## 2. The surfaces

- [x] 2.1 Update `src/instructions.md`, `skills/hakkiri/SKILL.md`, `README.md` and `AGENTS.md`, list the difference in the `vale-differences` spec, and add the patch changeset.  
  Verify `operator_docs` passes, and the changeset contains `hakkiri: patch`.  
  Evidence, 2026-11-11 (UTC): the full suite passed in a box with `operator_docs`, after clippy and rustfmt findings were fixed.

## 3. Integration

- [x] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.  
  Evidence, 2026-11-12 (UTC): PR 104 merged, and CI on main at 930ecee passed: `Build and test`, `Static checks` and `Verify`.
