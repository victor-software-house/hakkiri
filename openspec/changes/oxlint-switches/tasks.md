# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. The switch

- [ ] 1.1 Leave a TypeScript or JavaScript comment that opens with `oxlint` out of the tree.
  The word must end at a hyphen, a space or the end of the comment, so `// oxlintrc files.` stays prose.
  Verify `an_oxlint_switch_is_not_prose` passes and keeps only `oxlintrc files` and `A note`.
