# Tasks

Every compile, lint and test step runs in the pull request's CI.

## 1. The scope

- [ ] 1.1 Probe Vale at `66bbb85` for how its scopes read a link title.
  Verify the proposal records which scopes read it.
- [ ] 1.2 Add `Scope::Title` and the nodes for a link, an image and a definition.
  Verify the tests in `tests/select.rs` fail on the source before the change and pass after it.

## 2. The surfaces

- [ ] 2.1 Add the requirement to `scope-selection` and a patch changeset.
  Verify `openspec validate --all --strict` passes and the changeset names `hakkiri: patch`.

## 3. Integration

- [ ] 3.1 Verify the pull request's CI passes.
