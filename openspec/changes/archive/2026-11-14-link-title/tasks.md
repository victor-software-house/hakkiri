# Tasks

Every compile, lint and test step runs in the pull request's CI.

## 1. The scope

- [x] 1.1 Probe Vale at `66bbb85` for how its scopes read a link title.
  Verify the proposal records which scopes read it.  
  Evidence, 2026-11-11 (UTC): The proposal records the probe of Vale at `66bbb85`: only `raw` gave an alert at a title, over nine scopes.
- [x] 1.2 Add `Scope::Title` and the nodes for a link, an image and a definition.
  Verify the tests in `tests/select.rs` fail on the source before the change and pass after it.  
  Evidence, 2026-11-11 (UTC): [pull request 120](https://github.com/victor-software-house/hakkiri/pull/120) added `Scope::Title`, its nodes and the title tests of `tests/select.rs`, and the stack merged as `7aac10a` on 2026-11-13.

## 2. The surfaces

- [x] 2.1 Add the requirement to `scope-selection` and a patch changeset.
  Verify `openspec validate --all --strict` passes and the changeset names `hakkiri: patch`.  
  Evidence, 2026-11-14 (UTC): `.changeset/title-parts.md` names `hakkiri: patch`, and `openspec validate --all --strict` passes with this delta.

## 3. Integration

- [x] 3.1 Verify the pull request's CI passes.  
  Evidence, 2026-11-13 (UTC): The CI of the stack that [pull request 146](https://github.com/victor-software-house/hakkiri/pull/146) heads passed, and it merged as `7aac10a`.
