# Tasks

## 1. Repository

- [x] 1.1 Create the public repository with README and `AGENTS.md` in the first commit, and verify the commit is on `main`.
  - 2026-11-09 (UTC), [6f8aef2][c-first] is the first commit on `main` of [hakkiri][repo].
- [x] 1.2 Render the templates `rust-crate` type with `openspec` and `lint_prose`, and verify `.copier-answers.yml` records the tag.
  - 2026-11-09 (UTC), [20e8df5][c-render] records `_commit: v0.0.84`. The template's own check runs in CI (task 3.1).
- [x] 1.3 Lock the default, dev and ci mise profiles, and verify the pre-push `hooks:lockfile` check passes.
  - 2026-11-09 (UTC), the first push failed on a stale ci lock; [efe389a][c-ci-lock] fixed it, and the next push passed the hook.

## 2. Plan

- [x] 2.1 Queue the release 1 slices as HKR rows, and verify `qctl check` passes.
  - 2026-11-09 (UTC), HKR-001 to HKR-010 are queued and HKR-011 waits on the horizon; `qctl check --no-git` passed.
- [x] 2.2 Write this change's proposal, design and research, and verify `openspec validate --all --strict` passes.

## 3. Integration

- [ ] 3.1 Verify the pull request's CI passes: build, lint, tests, prose lint, OpenSpec and the Template check.
- [ ] 3.2 Add hakkiri to the ctl mani group and the ctl `AGENTS.md` table, and verify `mani check` lists it.

## Workflow follow-up

- Archive this change after the pull request merges, then archive HKR-001 with `qctl close-from-openspec`.

[repo]: https://github.com/victor-software-house/hakkiri
[c-first]: https://github.com/victor-software-house/hakkiri/commit/6f8aef2a4c537a6d2a30bb981f75be321316dd24
[c-render]: https://github.com/victor-software-house/hakkiri/commit/20e8df5191ddeed6a8c83c66c649e1ae27187fdd
[c-ci-lock]: https://github.com/victor-software-house/hakkiri/commit/efe389ae0099eebe5da77aceaa64ee2960b07ae6
