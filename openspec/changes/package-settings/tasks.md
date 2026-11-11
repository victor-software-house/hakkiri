# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
The code waits for `HKR-019`, because the settings include `vocab`, and for the operator's ruling on the three decisions in the [proposal][proposal].

## 1. Read the settings

- [ ] 1.1 Read `.ctl/hakkiri.yaml` of each loaded package with the schema of the project config, name the package and line in a fault, and refuse `packages` in it, in `src/config.rs`.  
  Verify `tests/package_settings.rs` holds a test for each scenario of the three requirements "A package may ship a settings file", "A package's settings file holds the keys of a project config" and the fetched-package scenario, and each fails on the code of main and passes with the change.
- [ ] 1.2 Layer the packages' `files` and `level` ahead of the project's, in package order, with the add-up rule for `styles` under one glob.  
  Verify the scenarios of "The project overrides a package" and "A project's styles join those of a package under one glob" each fail before and pass after.
- [ ] 1.3 Add up the `vocab` names after `HKR-019`.  
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
