# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
This change follows [`project-config`][config], which holds the grammar of the config, and [`package-fetch`][fetch], which keeps a package's `styles/config` folder.  
The key that names vocabularies waits for the operator's choice (design decision 2), and task 1.3 starts when it is made.

## 1. The reading

- [ ] 1.1 Probe Vale at `66bbb85` for what the design lists as not probed, and record every probe case with Vale's alerts in `tests/fixtures/vocabularies/`.  
  Verify `tests/fixtures/vocabularies/vale.tsv` holds the alerts of the `conditional`, `repetition` and `substitution` exceptions, `vocab: false` on `existence`, a plain term with a period and a phrase that wraps, with the Vale commit named in the file.
- [ ] 1.2 Add `src/vocabulary.rs`: find `config/vocabularies/<name>/` in a package folder, walk it, read each `accept.txt` and `reject.txt` as terms, and refuse an unknown name, two packages with one name and a term that is not a pattern.  
  Verify `tests/vocabularies.rs` fails before the change and passes after it, with a test for each scenario of the requirements on the folder, the line and the term.
- [ ] 1.3 Name the vocabularies in the config, and read the packages, then the vocabularies, then the styles.  
  Verify the scenarios of the `project-config` delta pass, and a config with no vocabulary key loads as before.

## 2. The checks

- [ ] 2.1 Add the accepted terms to the exceptions of `capitalization`, `conditional`, `existence`, `repetition` and `substitution` before they are rewritten and sorted, with `vocab: false` and `nonword` reading none, and to the converters of `capitalization`.  
  Verify the existence and capitalization scenarios pass, and each of the five checks has a test that fails before the change.
- [ ] 2.2 Add the accepted terms to `spelling` as whole words matched without regard to case, with a phrase masked.  
  Verify the spelling scenarios pass, and the test fails before the change.

## 3. The surfaces

- [ ] 3.1 Update `src/instructions.md`, `skills/hakkiri/SKILL.md`, `README.md` and `AGENTS.md`, narrow the vocabularies gap in the `vale-differences` spec, and add the patch changeset.  
  Verify `operator_docs` passes, the instructions name the vocabulary key, and the changeset names `hakkiri: patch`.

## 4. Integration

- [ ] 4.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.

[config]: ../../specs/project-config/spec.md
[fetch]: ../archive/2026-11-11-package-fetch/proposal.md
