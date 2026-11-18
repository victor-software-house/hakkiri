# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. The headers

- [x] 1.1 Leave a Shell comment that opens with `MISE` or `USAGE`, with or without brackets, out of the tree.
  Verify `comments_read_as_vale_reads_them_but_where_decided` passes with the `sh-directive.sh` input, which keeps only `november after` and `USAGE oscar`.
  Evidence, 2026-11-18 (UTC): the Build and test job of [hakkiri#179][pr-179] passes, `comment_languages` included.
- [x] 1.2 List the headers in the instructions beside the other directives.
  Verify `lint:prose` and `specs` pass.
  Evidence, 2026-11-18 (UTC): `mise run specs` passes 36 of 36, and the change's files report 0 alerts under ProseStyle v0.0.18.

[pr-179]: https://github.com/victor-software-house/hakkiri/pull/179
