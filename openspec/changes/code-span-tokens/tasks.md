# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The walk

- [x] 1.1 Give each word of a sentence a flag for a code span, and make `accepts` refuse it unless the token is negated.
  Verify the tests in `src/sequence/tests.rs` and `tests/sequence.rs` fail on the source before the change and pass after it.
  Evidence, 2026-11-12 (UTC): in a box the four new tests pass, and the comparison with Vale still passes.
  With the refusal turned off, the two unit tests of the sequence module fail.
  Vale 3.26.0 gives `'**** says' gives a thing speech.` for ``The call fails as `flag` says.`` and none for no span.
- [x] 1.2 Let a token that reads one word read the noun that a preposition before the span attaches it to, and start the run there.
  Verify the tests in `tests/sequence.rs` fail before the change and pass after it, for `` the notice for `cmd-7` says ``, `` the lock of the instance at `paths` says `` and a repeated token.
  Evidence, 2026-11-12 (UTC): in a box the two tests pass.
  Two other readings were measured and left out: the noun right before the span, and that with a cue for a stranded preposition and a determiner before the span.
- [x] 1.3 Diff the alerts of the nine repositories that run Vale, before and after.
  Verify the true alerts that the change loses are listed, and no false alert returns beyond those listed.
  Evidence, 2026-11-12 (UTC): at `--min-level suggestion` the change removes 72 alerts, 41 of `NounStack` and 31 of `ThingsSpeakNoun`, and adds 17.
  All 41 `NounStack` alerts removed are false.
  Of the 31 removed, 23 are false, from a code name or the plural noun `asks`, and 8 are true.
  Five of the 8 come back on the prose noun, as the lock that says, a marker that says twice, a rule that tells and a notice that says, among 7 new true alerts.
  Three stay lost: the scenario `Orphans after a crash` that said, the scenario `A blob file deleted by hand` that says, and the `UNIQUE` check on `session.dir` that agrees.
  Of the 16 `ThingsSpeakNoun` alerts added, 12 are true and 4 are false, the plural noun `asks` in `` the `/compact` asks ``.
  The one `NounStack` alert added is false and has no code span: the fix to the count of converted verbs restores it.

## 2. The surfaces

- [x] 2.1 Add the `sequence-check` and `vale-differences` requirements, the note in the instructions, the skill and `AGENTS.md`, and a patch changeset.
  Verify the OpenSpec validation, `lint:md` and `lint:prose` pass.
  Evidence, 2026-11-12 (UTC): the full `mise run verify` set passes in a box: `format`, `lint`, `test`, `doc`, `deny`, `deps`, `specs`, `specs:budget`, `lint:md`, `lint:prose` with 0 errors, `lint:typos`, `check:repo` and `check:versions`.
  Not run: `semver`, the advisory check of `deny`, and the workflow policy, which the pull request's CI runs.

## 3. Integration

- [ ] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
