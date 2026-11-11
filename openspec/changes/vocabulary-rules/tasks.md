# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
This change follows [`vocabularies`][vocabularies], which reads the lists and names the vocabularies in the config.

## 1. The rules

- [x] 1.1 Probe Vale at `66bbb85` for a term whose pattern holds `\W`, and record the probe cases of the requirements with Vale's alerts in `tests/fixtures/vocabulary-rules/vale.tsv`.  
  Verify the file holds the alerts of every scenario of `Terms` and `Avoid`, with the Vale commit named in it.  
  Evidence, 2026-11-11 (UTC): a Vale build of `66bbb85` ran in a box over `tests/fixtures/vocabulary-rules` with the nine configs in `vale/`, and gave 152 alerts.  
  Nothing in them contradicts the plan: the design lists what they add, including that Vale lowercases a term's escapes, so `data\Wset` flags `dataxset`.
- [x] 1.2 Build `Vale.Terms` and `Vale.Avoid` in a style `Vale`, with Vale's messages, levels, boundaries and recasing of the shown term.  
  Verify `tests/vocabulary_rules.rs` fails before the change and passes after it, against the alerts recorded in task 1.1.  
  Evidence, 2026-11-11 (UTC): `every_config_gives_the_alerts_vale_gives` failed on the source before the change, and passes after it for the nine configs.  
  With the glob rules left off, the recasing of the shown term left out, or a term matched as plain text and not as a pattern, in copies of the tree, it fails.  
  The last column of a match that spans lines is left out of it: hakkiri ends such a match on its first line, as it does for every rule.
- [x] 1.3 Run them where the style `Vale` or their name is listed, run a glob's rules on its files, and refuse `Vale` as a style when no named vocabulary holds a term.  
  Verify the scenarios of the requirements on where they run and on a glob's rules pass, and `styles: [Vale]` with no term is an error.  
  Evidence, 2026-11-11 (UTC): the tests for `styles: [Vale]` with no term, for a level on a rule that no term builds, and for a package style named `Vale` pass, and the first two passed before the change too.  
  The `levels`, `off`, `glob`, `globoff` and `twoglobs` configs of the parity test cover the settings by name and a glob's rules.

## 2. The surfaces

- [x] 2.1 Update `src/instructions.md`, `skills/hakkiri/SKILL.md`, `README.md` and `AGENTS.md`, drop the vocabulary-rules gap from the `vale-differences` spec, and add the patch changeset.  
  Verify `operator_docs` passes, the instructions name the style `Vale`, and the changeset names `hakkiri: patch`.  
  Evidence, 2026-11-11 (UTC): the full suite, 601 tests, passed in a box, and `operator_docs` with it.  
  The `vale-differences` delta removes the gap and adds the one intended difference.

## 3. Integration

- [ ] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.

[vocabularies]: ../archive/2026-11-11-vocabularies/tasks.md
