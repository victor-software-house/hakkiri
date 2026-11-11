# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
This change follows [`project-config`][config], which holds the grammar of the config, and [`package-fetch`][fetch], which keeps a package's `styles/config` folder.

## 1. The probe and the reading

- [x] 1.1 Probe Vale at `66bbb85` for the exceptions of each check, `vocab: false`, a plain term with a period, a phrase that wraps, and a glob's vocabulary, and record every alert in `tests/fixtures/vocabularies/vale.tsv`.  
  Verify the file holds the alerts of five configs over one package, with the Vale commit named in `tests/vocabularies.rs`.  
  Evidence, 2026-11-11 (UTC): a Vale build of `66bbb85` ran in a box over `tests/fixtures/vocabularies` with the five configs in `vale/`, and gave 293 alerts.  
  Nothing in them contradicts the plan, and the design lists what they add.
- [x] 1.2 Add `src/vocabulary.rs`: find `config/vocabularies/<name>/` in a package folder, walk it, read each `accept.txt` and `reject.txt` as terms, and refuse an unknown name, two packages with one name and a term that is not a pattern.  
  Verify `tests/vocabularies.rs` fails before the change and passes after it, with a test for each scenario of the requirements on the folder, the line and the term.  
  Evidence, 2026-11-11 (UTC): in a box, five of the seven tests in `tests/vocabularies.rs` failed before the change on the unknown key `vocab`, and a sixth failed on `vocab` read as a rule name.  
  All seven pass after it. The seventh, a config with no key, did not run before the change in its final form.  
  A term `C++` is read as a pattern with a possessive plus, so the test for a term that is not a pattern uses `C(`.
- [x] 1.3 Read `vocab` at the top level and in a glob, and read the packages, then the vocabularies, then the styles.  
  Verify the scenarios of the `project-config` delta pass, and a config with no `vocab` key loads as before.  
  Evidence, 2026-11-11 (UTC): the tests for an unknown name, two packages, a `vocab` that is no list, and a config with no key pass.  
  `a_vocabulary_of_a_fetched_package_is_found` in `tests/package_fetch.rs` failed before the change and passes after it.

## 2. The checks

- [x] 2.1 Add the accepted terms of the top level to the exceptions of `capitalization`, `conditional`, `existence`, `repetition` and `substitution` before they are rewritten and sorted, with `vocab: false` and `nonword` reading none, and to the converters of `capitalization`.  
  Verify the existence and capitalization scenarios pass, and each of the five checks has a test that fails before the change.  
  Evidence, 2026-11-11 (UTC): `every_config_gives_the_alerts_vale_gives` passes for the five configs against `vale.tsv`.  
  With the top level's terms taken out of the rules' exceptions, in a copy of the tree, it fails.  
  Not verified: each of the five checks alone, since that copy takes them out together.
- [x] 2.2 Add the accepted terms of the top level to `spelling` as whole words matched without regard to case, with a phrase masked.  
  Verify the spelling scenarios pass, and the test fails before the change.  
  Evidence, 2026-11-11 (UTC): the same test covers `Team.Spell` under each config, and fails with the accepted words and masks taken out of the `spelling` build.
- [x] 2.3 Hold a file's glob vocabularies in a set built once for each distinct list, and ask it of each match of an `existence`, `repetition`, `spelling` and `substitution` rule.  
  Verify the scenarios of the requirement on a glob's vocabularies pass against the alerts of `vale.tsv`, and each of the four checks has a test that fails before the change.  
  Evidence, 2026-11-11 (UTC): the same test passes for the `glob`, `union` and `two` configs, and fails when the run ignores a file's globs.  
  The rows of the rules named `Vale.<Name>.Terms` are left out of it, for `HKR-042`.  
  Not verified: each of the four checks alone, since that copy takes them out together.

## 3. The surfaces

- [x] 3.1 Update `src/instructions.md`, `skills/hakkiri/SKILL.md`, `README.md` and `AGENTS.md`, narrow the vocabularies gap in the `vale-differences` spec, and add the patch changeset.  
  Verify `operator_docs` passes, the instructions name the `vocab` key, and the changeset names `hakkiri: patch`.  
  Evidence, 2026-11-11 (UTC): the full suite, 568 tests, passed in a box, and `operator_docs` with it.

## 4. Integration

- [ ] 4.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.

[config]: ../../specs/project-config/spec.md
[fetch]: ../archive/2026-11-11-package-fetch/proposal.md
