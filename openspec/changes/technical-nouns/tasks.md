# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The word lists

- [x] 1.1 Add `nouns_of_technical_prose` to `src/tagger/corrections.rs`, with one word list for each place a noun is certain.
  Verify tagger tests pin each list, GUM stays at 92.53% or above, and the 7 true alerts of Vale that `NounStack` lacked are given.
  Evidence, 2026-11-12 (UTC): GUM is right on 52,298 of 56,516 tokens, 92.54%, as before the change, and the judged conflicts count 134 right against 133.
  `NounStack` gives all 16 true alerts of Vale, 14 of them with a span that runs through a link target.
- [x] 1.2 Keep in each list only the words whose sampled readings in that place are all nouns, and put the counts in the design.
  Verify two readers labelled 530 places of the six words in the nine repositories, and each case and list was run alone.
  Evidence, 2026-11-12 (UTC): after a determiner `check`, `prompt`, `thinking` and `fade` are nouns in 163 of 163 places, `round` in 41 of 50 and `split` in 40 of 50, so the last two left that list.
  The title case holds 5 true alerts of `NounStack` and the compound case 1, and the same cases for every word add 98 alerts of which a sample are false.
  GUM, the judged conflicts and every alert of the nine repositories are the same with the three lists.

## 2. Integration

- [ ] 2.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
