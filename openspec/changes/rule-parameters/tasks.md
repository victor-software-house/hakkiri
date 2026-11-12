# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. The probe

- [x] 1.1 Probe Vale at `66bbb85` for each parameter of each check type that has one, and record its alerts and its errors in `tests/fixtures/rule-parameters`.  
  Verify `vale.tsv` holds the alerts of every config, and `faults.tsv` holds one row for each fault with the message Vale gave, with the Vale commit named in the test.  
  Evidence, 2026-11-12 (UTC): a Vale build of `66bbb85` ran in a cloud box over two Markdown files and 24 configs, and gave 745 alerts.  
  The same build showed three things the ledger row did not hold: the key is `Style.Rule[param]`, a section's parameter applies to every file, and the last section's value wins everywhere.

## 2. The parameters

- [x] 2.1 Read `params` under a glob, read the rule again with them, and place each fault on the config's line.  
  Verify `tests/rule_parameters.rs` fails before the change and passes after it.  
  Evidence, 2026-11-12 (UTC): in the same box, `every_config_gives_the_alerts_vale_gives` gave Vale's alerts under all 24 configs.  
  The eight other tests passed.  
  They cover sixteen faults on their lines, a parameter that reaches only its glob, a later glob, an empty `styles` list, a package under the project, a vocabulary rule, a rule no glob runs, and four faults at once.

## 3. The surfaces

- [x] 3.1 Update `src/instructions.md`, the skill, `README.md` and `AGENTS.md`, and add the patch changeset.  
  Verify `operator_docs` passes and the changeset names `hakkiri: patch`.  
  Evidence, 2026-11-12 (UTC): in the same box, the whole `cargo test` passed and `cargo clippy --all-targets` passed.  
  On the authoring machine, `openspec validate --all --strict` and `qctl check --no-git` passed.  
  Not run here: `cargo deny`, `cargo machete`, the doc build and the musl clippy, which the pull request's CI runs.

## 4. Integration

- [ ] 4.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
