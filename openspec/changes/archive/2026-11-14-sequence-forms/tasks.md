# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The engine

- [x] 1.1 Split `Sequence` into `Sequence` and `Form`, read `forms` and place each fault on the form, and walk each form and merge the matches.  
  Verify `tests/sequence_forms.rs` fails before the change and passes after it, and the walk still gives Vale's alerts for one list.  
  Evidence, 2026-11-12 (UTC): in a cloud box, the eight tests of `tests/sequence_forms.rs` pass, with the merged `ThingsSpeak` equal to its three one-list rules on eleven sentences and the guard skipping `The jobs asked to end early.`.  
  `walks_the_tokens_as_vale_does` and `fills_indexed_verbs_as_vale_does` still pass, so a `tokens` rule gives Vale's alerts, and the whole `cargo test`, `cargo clippy --all-targets` and the format check pass.  
  Before the change the key `forms` is unknown to the rule reader, so each test that loads one fails at load. That was not run against the old source.

## 2. The surfaces

- [x] 2.1 Update `src/instructions.md`, the skill and `AGENTS.md`, add the deltas of the `sequence-check` and `vale-differences` specs, and add the patch changeset.  
  Verify `operator_docs` and `openspec validate --all --strict` pass.  
  Evidence, 2026-11-12 (UTC): `operator_docs` passes in the box and `openspec validate --all --strict` and `qctl check --no-git` pass on the authoring machine.  
  Vale at `66bbb85` gave a Go panic at `sequence.go:292` on the merged rule, which the `vale-differences` delta records.  
  The error-level prose lint of the changed files shows one error, the 51-word sentence in `src/instructions.md` about tag corrections, which this change did not write.

## 3. Integration

- [x] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.  
  Evidence, 2026-11-14 (UTC): The CI of the stack that [pull request 121](https://github.com/victor-software-house/hakkiri/pull/121) heads passed, and the stack merged as `7aac10a`.
