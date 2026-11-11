# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
The code builds on `HKR-019`, because the settings include `vocab`.  
The operator ruled on the three decisions of the [proposal][proposal] on 2026-11-11.

## 1. Read the settings

- [ ] 1.1 Read `.ctl/hakkiri.yaml` of each loaded package with the schema of the project config, and name the package and the line in a fault, in `src/config.rs`.  
  Verify `tests/package_settings.rs` holds a test for each scenario of "A package may ship a settings file" and "A package's settings file holds the keys of a project config".  
  Each test fails on the code of main and passes with the change.
- [ ] 1.2 Fetch the packages a package lists, with the cycle and depth checks, once per `url` and `sha256`, in `src/config.rs`.  
  Verify the scenarios of "A package may list packages" each fail before and pass after, the nested fetch served by the local test server of `tests/package_fetch.rs`.
- [ ] 1.3 Layer the packages' `files` and `level` ahead of the project's, in package order, with the add-up rule for `styles` under one glob.  
  Verify the scenarios of "The project overrides a package" and "A project's styles join those of a package under one glob" each fail before and pass after.
- [ ] 1.4 Add up the `vocab` names after `HKR-019`.  
  Verify the two scenarios of "Package vocabularies and the project's add up" pass.

## 2. Record

- [ ] 2.1 Update `--help`, `src/instructions.md`, the skill, `AGENTS.md` and the specs, and add a patch changeset.  
  Verify `openspec validate --all --strict` and the budget check pass.
- [ ] 2.2 Gate in a throwaway Linux box.  
  Verify test, lint, format, doc, deny and deps pass.

## 3. Gate

- [ ] 3.1 Pull request CI passes on the head of the pull request.  
  Verify the checks of the head commit pass in the run on the pull request.

[proposal]: proposal.md
