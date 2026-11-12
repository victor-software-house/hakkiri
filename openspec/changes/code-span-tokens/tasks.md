# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The walk

- [x] 1.1 Give each word of a sentence a flag for a code span, and make `accepts` refuse it unless the token is negated.
  Verify the tests in `src/sequence/tests.rs` and `tests/sequence.rs` fail on the source before the change and pass after it.
  Evidence, 2026-11-12 (UTC): in a box the four new tests pass, and the comparison with Vale still passes.
  With the refusal turned off, the two unit tests of the sequence module fail.
  Vale 3.26.0 gives `'**** says' gives a thing speech.` for ``The call fails as `flag` says.`` and none for no span.
- [x] 1.2 Diff the alerts of the nine repositories that run Vale, before and after.
  Verify no removed alert is a true alert whose noun is prose, or record the ones that are.
  Evidence, 2026-11-12 (UTC): at `--min-level suggestion` the change removes 72 alerts and adds none, 41 of `NounStack` and 31 of `ThingsSpeakNoun`.
  All 41 `NounStack` alerts are false, each with a code span and fewer than four prose nouns.
  Of the 31, 23 are false, from a code name or the plural noun `asks`.
  Eight are true and lost, and each names a prose noun that speaks with a code span before the verb.
  They are a scenario that says, a notice that says, a marker that says, a check that agrees, a lock that says and a rule that tells.

## 2. The surfaces

- [x] 2.1 Add the `sequence-check` and `vale-differences` requirements, the note in the instructions, the skill and `AGENTS.md`, and a patch changeset.
  Verify the OpenSpec validation, `lint:md` and `lint:prose` pass.
  Evidence, 2026-11-12 (UTC): the full `mise run verify` set passes in a box: `format`, `lint`, `test`, `doc`, `deny`, `deps`, `specs`, `specs:budget`, `lint:md`, `lint:prose` with 0 errors, `lint:typos`, `check:repo` and `check:versions`.
  Not run: `semver`, the advisory check of `deny`, and the workflow policy, which the pull request's CI runs.

## 3. Integration

- [ ] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
