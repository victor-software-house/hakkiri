# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. The section comment

- [ ] 1.1 Leave a Swift line comment that opens with `MARK:` out of the tree.
  The word must be followed by a colon, so `// MARKER words.` stays prose.
  Verify `a_swift_mark_is_not_prose` passes and keeps only `MARKER words`.
