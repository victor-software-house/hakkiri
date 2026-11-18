# Design

## Decisions

### 1. The tagger reads the nouns that technical prose uses where the model reads another part of speech

The model reads `split`, `round`, `check`, `prompt`, `fade` and `thinking` as one other tag in these runs:

- `the host load split pull request`
- `the mask round pull request`
- `the peer user id check`
- `the prompt template pull request`
- `the fade removal pull request`
- `the thinking request fields decision`

A run of four nouns lost one, and `NounStack` lost 7 true alerts of Vale.  
The model lists some of these words with one tag and never scores them.  
For `prompt`, the score of a noun is 25 points under an adjective.  
So no margin of the model separates `prompt` from `final`.

Three lists in `src/tagger/corrections.rs` contain the words.  
Each list covers the one place where the sampled readings show a noun and nothing else:

- `NOUNS_AFTER_DETERMINER` contains `check`, `prompt`, `thinking` and `fade`.  
  They read as nouns after a determiner, a possessive, or an adjective that is no participle.
- `NOUNS_IN_TITLE` contains `prompt`, `round` and `split`.  
  They read as nouns after a noun when the nouns after them run up to `pull request`.
- `NOUNS_ENDING_COMPOUND` contains `check`.  
  It reads as a noun when a base form follows two singular nouns before a preposition or a stop.

Each place has its own word list, and that list is the evidence for it.  
At first, all six words were listed for all three places.  
The counts showed that `round` and `split` were not always nouns after a determiner (see the measures).

These other ideas were measured and dropped:

- a model margin;
- a rule for a word between nouns;
- a rule for any adjective before three nouns;
- a title or compound rule for every word.

Each cost tokens of the Georgetown University Multilayer (GUM) corpus, or gave false alerts such as `record then places`, `names a pull request` and `base the link-scenarios pull request`.

A rule that read a hyphenated word as an adjective lost 6 tokens of GUM and was dropped.  
The definition of `NounStack` contains that reading.  
The rule's pattern accepts a hyphen, and 12 alerts of `NounStack` that are false by the definition rest on it.

## Measures

`N` and `Fix` are the builds of the measures of `code-span-tokens`.  
`N` lacked 7 true alerts of Vale, and `Fix` gives all of them:

- `peer user id check`
- `mask round pull request`
- `host load split pull request`
- `load split pull request`
- `fade removal pull request`
- `prompt template pull request`
- `thinking request fields decision`

### The word lists

Two readers labelled the part of speech at 530 places where the six words occur.  
The places come from the prose and the comments of the same nine repositories, taken from 17,465 places.  
There are 100 for each word and 30 for `fade`.

The readers differed on one place, `split` in a file name.  
Before the correction, the model is wrong on `prompt` in 78 of 94 sampled places, and on `thinking` and `round` in about half.

| Word | Places | Noun | Not a noun | Noun after a determiner |
|:--|--:|--:|--:|--:|
| `check` | 100 | 83 | 17 | 50 of 50 |
| `fade` | 30 | 29 | 1 | 13 of 13 |
| `prompt` | 100 | 100 | 0 | 50 of 50 |
| `round` | 100 | 79 | 21 | 41 of 50 |
| `split` | 99 | 49 | 50 | 40 of 50 |
| `thinking` | 100 | 99 | 1 | 50 of 50 |

GUM contains 21 places of the six words.  
One is a noun, `In his thinking`, after a possessive, which is a place of the first list.  
The others are verbs, as in `to check` and `I was thinking`, or a preposition.  
So GUM gives no reading against the three places of the lists, and it gives none for a noun outside them.

The places not after a determiner are where the word is a verb or a modifier.  
Examples are `to check`, `split` as a participle, and `the round trip`, which both readers read as an adjective.

Each case ran on the same 530 places, and on the nine repositories with its word list changed:

| Change | Places fixed | Places harmed | Alerts of the nine repositories |
|:--|--:|--:|:--|
| All six words after a determiner | 84 | 6 | none lost |
| `round` and `split` left out | 61 | 0 | none lost |
| Title case off | | | 8 lost, 5 true of `NounStack` |
| Compound case off | | | 1 lost, a true alert of Vale |
| Title and compound cases for every word | | | 98 more alerts of `NounStack`, such as `names a pull request` |
| Title case for `prompt`, `round` and `split`; compound case for `check` | | | none lost, none added |

The six harmed places read `the round trip` and `the split DNS` as nouns.  
Both readers read an adjective or a participle there.  
GUM and the judged conflicts stay at 92.54% and 134 under every row.

The model stays wrong on `prompt`, `thinking` and `round` in places that no list covers, such as `system prompt` and `thinking blocks`.  
No alert of the labelled sets depends on them.  
So a rule for them waits for an alert that needs it, with the labels to prove it.
