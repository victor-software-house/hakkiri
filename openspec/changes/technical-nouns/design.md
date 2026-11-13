# Design

## Decisions

### 1. The tagger reads the nouns that technical prose uses where the model reads another part of speech

The model reads `split`, `round`, `check`, `prompt`, `fade` and `thinking` as one other tag in `the host load split pull request`, `the mask round pull request`, `the peer user id check`, `the prompt template pull request`, `the fade removal pull request` and `the thinking request fields decision`.
A run of four nouns lost one, and `NounStack` lost 7 true alerts of Vale.
The model lists some of these words with one tag and never scores them, and for `prompt` the score of a noun is 25 points under an adjective, so no margin of the model separates `prompt` from `final`.
Three lists in `src/tagger/corrections.rs` hold the words, each for the one place where the sampled readings show a noun and nothing else:

- `NOUNS_AFTER_DETERMINER` holds `check`, `prompt`, `thinking` and `fade`, read as nouns after a determiner, a possessive, or an adjective that is no participle.
- `NOUNS_IN_TITLE` holds `prompt`, `round` and `split`, read as nouns after a noun when the nouns after them run up to `pull request`.
- `NOUNS_ENDING_COMPOUND` holds `check`, read as a noun when a base form follows two singular nouns before a preposition or a stop.

The word list of a place is the evidence for that place.
The first version listed all six words for all three places, and the counts showed that `round` and `split` did not hold after a determiner (see the measures).
A model margin, a rule for a word between nouns, a rule for any adjective before three nouns, and a title or compound rule for every word were measured and dropped.
Each cost GUM tokens or gave false alerts such as `record then places`, `names a pull request` and `base the link-scenarios pull request`.
A rule that read a hyphenated word as an adjective, which the definition of `NounStack` holds, lost 6 tokens of GUM and was dropped.
The rule's pattern accepts a hyphen, and 12 alerts of `NounStack` that are false by the definition rest on it.

## Measures

`N` and `Fix` are the builds of the measures of `code-span-tokens`.
The 7 true alerts of Vale that `N` lacked name `peer user id check`, `mask round pull request`, `host load split pull request`, `load split pull request`, `fade removal pull request`, `prompt template pull request` and `thinking request fields decision`, and `Fix` gives all of them.

### The word lists

Two readers labelled the part of speech of 530 places of the six words in the prose and the comments of the same nine repositories, 100 for each word and 30 for `fade`, taken from 17,465 places.
They differed on one place, `split` in a file name.
The model is wrong on `prompt` in 78 of 94 sampled places before the correction, and `thinking` and `round` in about half.

| Word | Places | Noun | Not a noun | Noun after a determiner |
|:--|--:|--:|--:|--:|
| `check` | 100 | 83 | 17 | 50 of 50 |
| `fade` | 30 | 29 | 1 | 13 of 13 |
| `prompt` | 100 | 100 | 0 | 50 of 50 |
| `round` | 100 | 79 | 21 | 41 of 50 |
| `split` | 99 | 49 | 50 | 40 of 50 |
| `thinking` | 100 | 99 | 1 | 50 of 50 |

GUM holds 21 places of the six words.
One is a noun, `In his thinking`, after a possessive, which is a place of the first list; the others are verbs, as in `to check` and `I was thinking`, or a preposition.
So GUM gives no reading against the three places of the lists, and it gives none for a noun outside them.
The places not after a determiner are where the word is a verb or a modifier, as in `to check`, `split` as a participle, and `the round trip` read as an adjective by both readers.

Each case ran on the same 530 places, and on the nine repositories with its word list changed:

| Change | Places fixed | Places harmed | Alerts of the nine repositories |
|:--|--:|--:|:--|
| All six words after a determiner | 84 | 6 | none lost |
| `round` and `split` left out | 61 | 0 | none lost |
| Title case off | | | 8 lost, 5 true of `NounStack` |
| Compound case off | | | 1 lost, a true alert of Vale |
| Title and compound cases for every word | | | 98 more alerts of `NounStack`, such as `names a pull request` |
| Title case for `prompt`, `round` and `split`; compound case for `check` | | | none lost, none added |

The six harmed places read `the round trip` and `the split DNS` as nouns, where both readers read an adjective or a participle.
GUM and the judged conflicts stay at 92.54% and 134 under every row.
The model stays wrong on `prompt`, `thinking` and `round` in the places that no list names, such as `system prompt` and `thinking blocks`.
No alert of the labelled sets depends on them, so a rule for them waits for an alert that needs it, with the labels to prove it.
