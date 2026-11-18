# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
This change follows [`vocabularies`][vocabularies], which reads the lists and names the vocabularies in the config.

## 1. The rules

- [x] 1.1 Probe Vale at `66bbb85` for a term whose pattern contains `\W`.  
  Record the probe cases of the requirements and Vale's alerts in `tests/fixtures/vocabulary-rules/vale.tsv`.  
  Verify the file contains the alerts of every scenario of `Terms` and `Avoid`, and gives the Vale commit.

  Evidence, 2026-11-11 (UTC): a Vale build of `66bbb85` ran in a box over `tests/fixtures/vocabulary-rules`.  
  It used the nine configs in `vale/`, and gave 152 alerts.  
  Nothing in them contradicts the plan, and the design lists what they add.  
  One addition: Vale lowercases a term's escapes, so `data\Wset` flags `dataxset`, where hakkiri flags `DATA SET`.
- [x] 1.2 Build `Vale.Terms` and `Vale.Avoid` in a style `Vale`, with Vale's messages, levels, boundaries and recasing of the shown term.  
  Verify `tests/vocabulary_rules.rs` fails before the change and passes after it, against the alerts recorded in task 1.1.

  Evidence, 2026-11-11 (UTC): `every_config_gives_the_alerts_vale_gives` failed on the source before the change, and passes after it for the nine configs.

  It fails on tree copies that have any of three faults.  
  One leaves the glob rules off, and one leaves out the recasing of the shown term.  
  The third matches a term as plain text and not as a pattern.

  The last column of a match that spans lines is left out of the test.  
  hakkiri ends such a match on its first line, as it does for every rule.
- [x] 1.3 Run them where the style `Vale` or their name is listed, and run a glob's rules on its files.  
  Reject `Vale` as a style when no listed vocabulary contains a term.  
  Verify the scenarios of the requirements on where they run and on a glob's rules pass.  
  Verify also that `styles: [Vale]` with no term is an error.

  Evidence, 2026-11-11 (UTC): three tests pass, and the first two passed before the change too.  
  They cover `styles: [Vale]` with no term, a level on a rule that no term builds, and a package style called `Vale`.  
  The `levels`, `off`, `glob`, `globoff` and `twoglobs` configs of the parity test cover the settings by name and a glob's rules.

## 2. The surfaces

- [x] 2.1 Update `src/instructions.md`, `skills/hakkiri/SKILL.md`, `README.md` and `AGENTS.md`.  
  Drop the vocabulary-rules gap from the `vale-differences` spec, and add the patch changeset.  
  Verify `operator_docs` passes, the instructions describe the style `Vale`, and the changeset contains `hakkiri: patch`.

  Evidence, 2026-11-11 (UTC): the full suite, 601 tests, passed in a box, and `operator_docs` with it.  
  The `vale-differences` delta removes the gap and adds the one intended difference.

## 3. Integration

- [x] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.

  Evidence, 2026-11-11 (UTC): [PR 92][pr-92] sits in a stack, where only the Kody check ran on its head, with 0 open findings.  
  The box gate passed on the rebased tree, 601 tests.  
  It passed again on this archive's tree after the escape change, and the Mac gate passed on both.

  No other check has run on this change alone.

[vocabularies]: ../2026-11-11-vocabularies/tasks.md
[pr-92]: https://github.com/victor-software-house/hakkiri/pull/92
