# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. The alt node

- [x] 1.1 Add `html_text::inline_tag_nodes`, which gives the titles and the alt text of an inline tag, and call it for inline HTML in `src/markdown.rs`.  
  Verify `the_alt_text_of_an_inline_img_is_an_alt_block` in `tests/select.rs` passes in CI.  
  Evidence, 2026-11-14 (UTC): The CI of [pull request 170](https://github.com/victor-software-house/hakkiri/pull/170) passed at `061a8ef`, `the_alt_text_of_an_inline_img_is_an_alt_block` among its tests, and the Markdown corpus dropped 16 rows that only Vale gave.
- [x] 1.2 Add the `html-block-text` requirement, remove the `vale-differences` gap, and add the patch changeset.  
  Verify `openspec validate --all --strict` passes.  
  Evidence, 2026-11-14 (UTC): `openspec validate --all --strict` passes, and `.changeset/inline-img-alt.md` names `hakkiri: patch`.

## 2. Integration

- [x] 2.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.  
  Evidence, 2026-11-14 (UTC): The CI of [pull request 170](https://github.com/victor-software-house/hakkiri/pull/170) passed at `061a8ef`.
