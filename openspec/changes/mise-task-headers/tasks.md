# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. The headers

- [ ] 1.1 Leave a Shell comment that opens with `MISE` or `USAGE`, with or without brackets, out of the tree.  
  Verify `comments_read_as_vale_reads_them_but_where_decided` passes with the `sh-directive.sh` input, which keeps only `november after` and `USAGE oscar`.
- [ ] 1.2 List the headers in the instructions beside the other directives.  
  Verify `lint:prose` and `specs` pass.
