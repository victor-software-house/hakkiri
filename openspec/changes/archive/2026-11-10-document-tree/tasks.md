# Tasks

## 1. Markdown and MDX

- [x] 1.1 Add the tree type and parse Markdown with markdown-rs.  
  Verify the snapshot tests for headings, nested lists and tables pass in CI.
  - 2026-11-10 (UTC), `mise run test` ran in a throwaway Linux box.  
    The heading, nested list and table snapshot tests pass among 71 tests.  
    CI has not run yet (task 3.1).
- [x] 1.2 Parse MDX, and verify the MDX snapshot test passes in CI.
  - 2026-11-10 (UTC), the MDX snapshot test and the JSX element test pass in the same box run.  
    A JSX tag or an expression is a gap in the paragraph around it, which stays one node.

## 2. Source comments

- [x] 2.1 Find Rust, TypeScript and JavaScript comments with tree-sitter, and read each as Markdown.  
  Verify the snapshot tests for block comments, joined doc comments and strings holding URLs pass in CI.
  - 2026-11-10 (UTC), the block comment, joined doc comment and string literal snapshot tests pass in the same box run.
- [x] 2.2 Map spans inside comments back to the file, and verify the indented doc-comment link test passes in CI.
  - 2026-11-10 (UTC), the indented doc-comment link test passes in the same box run: the `link` node spans the link in the file line.
- [x] 2.3 Leave JSDoc tag names out of prose, and verify the JSDoc snapshot test passes in CI.
  - 2026-11-10 (UTC), the JSDoc snapshot test and the parameter tag test pass in the same box run.

## 3. Integration

- [x] 3.1 Verify the pull request's CI passes: build, lint, tests, prose lint, OpenSpec and the Template check.
  - 2026-11-10 (UTC), [Verify][run-pr7] passed on [PR 7][pr7] at `ab0f0b7`.  
    The Template check passed at v0.0.88.  
    Every Kody thread was answered and resolved, and the pull request merged at `94ae55e`.

## Workflow follow-up

- Archive this change after the pull request merges, then archive `HKR-003` with `qctl close-from-openspec`.

[pr7]: https://github.com/victor-software-house/hakkiri/pull/7
[run-pr7]: https://github.com/victor-software-house/hakkiri/actions/runs/42782271493
