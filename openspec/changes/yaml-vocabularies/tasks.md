# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. The probe

- [x] 1.1 Probe Vale at `66bbb85` for each entry shape and each fault of a YAML vocabulary, and record its alerts and error lines in `tests/fixtures/yaml-vocabularies`.  
  Verify `vale.tsv` holds the alerts of the four configs and `errors.tsv` the error line of each of the twelve faults, with the Vale commit named in the test.  
  Evidence, 2026-11-12 (UTC): a Vale build of `66bbb85` ran in a cloud box over two Markdown files, four alert configs and nineteen vocabularies, and gave 135 alerts and twelve error lines.  
  The probe shows that Vale reads `regex: yes` as true, skips a null entry, ignores a key the file does not define, and reads the affix file from the dictionaries folders and `$DICPATH`.

## 2. The reader

- [x] 2.1 Read `vocab.yml` and `vocab.yaml` in `src/vocabulary/yaml.rs`, join their patterns with those of the text lists, and look the affix file up in the packages.  
  Verify `tests/yaml_vocabularies.rs` fails before the change and passes after it.  
  Evidence, 2026-11-12 (UTC): in the same box, the six tests of `tests/yaml_vocabularies.rs` pass: the four configs give Vale's 135 alerts, each of the twelve faults names the file and the line of Vale's error, and three further tests cover several faults at once, an unknown key and the affix lookup.  
  The first run failed because the `all` config of hakkiri listed no style `Vale`.

## 3. The surfaces

- [x] 3.1 Update `src/instructions.md`, the skill, `README.md` and `AGENTS.md`, point `HKR-041` at this change, and add the patch changeset.  
  Verify `operator_docs` passes and the changeset names `hakkiri: patch`.  
  Evidence, 2026-11-12 (UTC): in the same box, the whole `cargo test` ran with every test binary passing and `cargo clippy --all-targets` passed.  
  On the authoring machine, `openspec validate --all --strict` and `qctl check --no-git` passed.  
  Not run here: the formatting nightly `cargo fmt`, which the commit hook runs as a check, `cargo deny`, `cargo machete`, the doc build and the musl clippy, which the pull request's CI runs.

## 4. Integration

- [ ] 4.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
