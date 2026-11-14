# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. The alt node

- [ ] 1.1 Add `html_text::inline_tag_nodes`, which gives the titles and the alt text of an inline tag, and call it for inline HTML in `src/markdown.rs`.  
  Verify `the_alt_text_of_an_inline_img_is_an_alt_block` in `tests/select.rs` passes in CI.
- [ ] 1.2 Add the `html-block-text` requirement, remove the `vale-differences` gap, and add the patch changeset.  
  Verify `openspec validate --all --strict` passes.

## 2. Integration

- [ ] 2.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
