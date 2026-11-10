# Tasks

## 1. Markdown and MDX

- [ ] 1.1 Add the tree type and parse Markdown with markdown-rs.  
  Verify the snapshot tests for headings, nested lists and tables pass in CI.
- [ ] 1.2 Parse MDX, and verify the MDX snapshot test passes in CI.

## 2. Source comments

- [ ] 2.1 Find Rust, TypeScript and JavaScript comments with tree-sitter, and read each as Markdown.  
  Verify the snapshot tests for block comments, joined doc comments and strings holding URLs pass in CI.
- [ ] 2.2 Map spans inside comments back to the file, and verify the indented doc-comment link test passes in CI.
- [ ] 2.3 Leave JSDoc tag names out of prose, and verify the JSDoc snapshot test passes in CI.

## 3. Integration

- [ ] 3.1 Verify the pull request's CI passes: build, lint, tests, prose lint, OpenSpec and the Template check.

## Workflow follow-up

- Archive this change after the pull request merges, then archive `HKR-003` with `qctl close-from-openspec`.
