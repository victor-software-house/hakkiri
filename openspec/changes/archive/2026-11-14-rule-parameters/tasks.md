# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. The probe

- [x] 1.1 Probe Vale at `66bbb85` for each parameter of each check type that has one.  
  Record its alerts and its errors in `tests/fixtures/rule-parameters`.  
  Verify `vale.tsv` contains the alerts of every config.  
  Verify `faults.tsv` has one row for each fault with the message Vale gave, and the test states the Vale commit.
  - Evidence, 2026-11-12 (UTC): a Vale build of `66bbb85` ran in a cloud box over two Markdown files and 24 configs, and gave 745 alerts.  
    The same build showed three things the ledger row did not contain.  
    The key is `Style.Rule[param]`, a section's parameter applies to every file, and the last section's value wins everywhere.

## 2. The parameters

- [x] 2.1 Read `params` under a glob, read the rule again with them, and place each fault on the config's line.  
  Verify `tests/rule_parameters.rs` fails before the change and passes after it.
  - Evidence, 2026-11-12 (UTC): in the same box, `every_config_gives_the_alerts_vale_gives` gave Vale's alerts under all 24 configs.  
  - The eight other tests passed.  
    They cover sixteen faults on their lines, a parameter that reaches only its glob, a later glob, and an empty `styles` list.  
    They also cover a package under the project, a vocabulary rule, a rule no glob runs, and four faults at once.

## 3. The surfaces

- [x] 3.1 Update `src/instructions.md`, the skill, `README.md` and `AGENTS.md`, and add the patch changeset.  
  Verify `operator_docs` passes and the changeset contains `hakkiri: patch`.
  - Evidence, 2026-11-12 (UTC): in the same box, the whole `cargo test` passed and `cargo clippy --all-targets` passed.  
    On the authoring machine, `openspec validate --all --strict` and `qctl check --no-git` passed.  
    Not run here: `cargo deny`, `cargo machete`, the doc build and the musl clippy, which the pull request's CI runs.

- [x] 3.2 Record where a `consistency` alert goes in a multi-line paragraph, with a fixture that pins both tools.  
  hakkiri puts it on the last match, and Vale on the first line that contains the text.  
  Verify `a_paragraph_of_lines_puts_the_alert_at_the_last_match` passes, and `openspec validate --all --strict` passes.
  - Evidence, 2026-11-12 (UTC): Vale at `66bbb85` ran under `nonword: true` on a paragraph of two lines.  
    The lines are `The adviser spoke first.` and `Two advisors met two advisers.`  
    It gave line 1, columns 5 to 11, and the test pins hakkiri at line 2, column 22.  
    The test passed in a cloud box, with `cargo clippy --all-targets` and the format check.
  - Vale's source shows why: `loc` stores the last match of the block.  
    `makeAlert` then searches the block for the matched text, which lands on its first occurrence.

## 4. Integration

- [x] 4.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.  
  Evidence, 2026-11-14 (UTC): The CI of the stack that [pull request 121](https://github.com/victor-software-house/hakkiri/pull/121) heads passed, and the stack merged as `7aac10a`.
