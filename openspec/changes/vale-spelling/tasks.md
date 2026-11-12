# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The probe

- [x] 1.1 Probe Vale at `66bbb85` for `Vale.Spelling` over Markdown, MDX, Rust and TypeScript files under 11 configs, and record its alerts in `tests/fixtures/vale-spelling`.  
  Verify `vale.tsv` holds the alerts of every config that gives one.  
  Evidence, 2026-11-12 (UTC): a Vale build of `66bbb85` ran in a cloud box and gave 880 alerts.  
  It showed that a dictionary in `config/dictionaries` adds to the built-in one, that `dictionaries` replaces it, and that the word lists are every `.txt` file below `config/ignore`.

## 2. The rule

- [x] 2.1 Add `vale_spelling`, read the folders of every package for the rule, and put it in the style `Vale`.  
  Verify `tests/vale_spelling.rs` fails before the change and passes after it.  
  Evidence, 2026-11-12 (UTC): in a cloud box, the five tests of `tests/vale_spelling.rs` pass, with hakkiri giving the 880 alerts of Vale under 11 configs except the 36 alerts of `departures.tsv`.  
  Before the change the style `Vale` holds no `Vale.Spelling`, so the config `off` fails to name it and the other configs give no spelling alert. That was not run against the old source.  
  The whole `cargo test` and `cargo clippy --all-targets` pass, and the format check passes.

## 3. The surfaces

- [x] 3.1 Update `src/instructions.md`, the skill and `AGENTS.md`, add the deltas of the `spelling-check` and `vale-differences` specs, turn the rule off in the hakkiri configs of the other `Vale` fixtures, and add the patch changeset.  
  Verify `operator_docs` and `openspec validate --all --strict` pass.  
  Evidence, 2026-11-12 (UTC): `operator_docs` passes in the box, and `openspec validate --all --strict` and `qctl check --no-git` pass on the authoring machine.  
  The fixtures of `builtin_repetition`, `vocabulary_rules`, `yaml_vocabularies` and `rule_parameters` failed until their hakkiri configs turned the rule off, as their Vale configs do.

## 4. Integration

- [ ] 4.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
