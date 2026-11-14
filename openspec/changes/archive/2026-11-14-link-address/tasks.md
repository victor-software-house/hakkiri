# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The reader

- [x] 1.1 Read the definitions of a file and mask a link whose text is its address in the text around it.  
  Verify `a_link_whose_text_is_its_address_counts_no_word` in `tests/metric.rs` passes, and the word-count probe of [HKR-063][ledger] shows no difference left that the link text causes.
- [x] 1.2 Compare the text with the address as goldmark writes it into the `href`.  
  Verify `a_link_is_its_address_only_when_its_text_is_the_href_vale_reads` in `tests/metric.rs` passes, with the word counts of Vale 66bbb85.

## 2. Integration

- [x] 2.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.  
  Evidence, 2026-11-14 (UTC): The CI of the stack that [pull request 121](https://github.com/victor-software-house/hakkiri/pull/121) heads passed, and the stack merged as `7aac10a`.

[ledger]: ../../../../tasks.yaml
