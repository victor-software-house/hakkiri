# Tasks

## 1. Segments

- [x] 1.1 Give each node segments, and join them into its text.  
  Verify the paragraph-over-two-line-comments test passes in CI.
  - 2026-11-10 (UTC), the test passes in a throwaway Linux box with `mise run test`, in [d315e7b][c-segments].  
    CI has not run yet (task 2.1).
- [x] 1.2 Leave code, HTML, link definitions and MDX syntax as gaps between segments.  
  Verify the HTML-in-a-paragraph and expression-in-a-paragraph tests pass in CI.
  - 2026-11-10 (UTC), both tests pass in the same kind of box run, in [ae31a1a][c-gaps].
- [x] 1.3 Make each Markdown line one segment, without the line ending, the indentation or the `>` markers.  
  Verify the quote-paragraph snapshot and test pass in CI.
  - 2026-11-10 (UTC), both pass in the same kind of box run, in [02f1676][c-lines].
- [x] 1.4 Leave heading syntax out of the segments.  
  Verify the heading-syntax snapshot passes in CI.
  - 2026-11-10 (UTC), the snapshot and the heading test pass among 69 tests in the same kind of box run, in [eb6115b][c-syntax].
- [x] 1.5 Make a block quote, list item and table row read as their content.  
  Verify the quote and list tests pass in CI.
  - 2026-11-10 (UTC), both pass in the same box run, in [eb6115b][c-syntax].  
    A container span now starts at its first segment.

## 2. Integration

- [x] 2.1 Verify the pull request's CI passes: build, lint, tests, prose lint, OpenSpec and the Template check.
  - 2026-11-10 (UTC), [Verify][run-pr7] passed on [PR 7][pr7] at `ab0f0b7`.  
    The Template check passed at v0.0.88.  
    Every Kody thread was answered and resolved, and the pull request merged at `94ae55e`.

## Workflow follow-up

- Archive this change after the pull request merges, then archive its queue row with `qctl close-from-openspec`.

[c-segments]: https://github.com/victor-software-house/hakkiri/commit/d315e7bb0373c4a8a1bb08a6b8ea42b8bc3af644
[c-gaps]: https://github.com/victor-software-house/hakkiri/commit/ae31a1ac38d6a4657c5fdc8999b760824d4184d7
[c-lines]: https://github.com/victor-software-house/hakkiri/commit/02f167627237fe417271074a75b41ae7313eb0cd
[c-syntax]: https://github.com/victor-software-house/hakkiri/commit/eb6115b9886b3bb509c6a17e3457a9548fd10584
[pr7]: https://github.com/victor-software-house/hakkiri/pull/7
[run-pr7]: https://github.com/victor-software-house/hakkiri/actions/runs/42782271493
