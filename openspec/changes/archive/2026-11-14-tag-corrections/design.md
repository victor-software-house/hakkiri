# Design

## Decisions

### 1. Keep the model and correct the tags afterwards

The older model was right more often on the conflicts of our documents, and wrong more often on general English.
It also needs the `bytes` feature template, which hakkiri does not read.

The alternative was to ship that model.
It needs a second reader, and it scores 1.9 points lower on the Georgetown University Multilayer (GUM) corpus: 90.26% against 92.16%.

### 2. Each correction is a rule with a reason, in one file

`corrections.rs` contains the rules and what each one fixes, so a reader sees why a tag differs from the model's.
The alternative was a second model trained on our documents.
It has no corpus to train on, and no way to review its choices.

### 3. The rules read the words and the tags of one sentence only

They run after the model, so a corrected tag is never fed back into the model's context.
A rule that reads the tag before a word reads the corrected one.

### 4. A title keeps the model's tags

The capitalization check tags a title through `tag_treebank` and ports Vale's result, so the rules do not run there.
The alternative was to correct titles too.
That changes a check that no measurement here covers.

### 5. Words in capitals are names, except units or function words

The model reads an acronym such as `CSS` or `SSH` as a noun, an adjective or a verb.
The old model reads it as a proper noun.

The model reads a heading such as `THE ANSWER` correctly.
GUM has capital headings of ordinary words such as `THE` and `OF`, so a function word keeps its tag.
The alternative was a list of acronyms, which never ends.

### 6. A plural noun is a verb when an object follows

The model reads `reads` in `The freshness harness reads stdout.` as `NNS`.
It reads any verb in `s` after a compound of nouns that way.
A noun before it ends the noun phrase.
A determiner, a possessive pronoun, a number or a pronoun after it starts the object.

A noun, an adjective or a preposition after it gives a verb only when no other word of the sentence is a verb.
A compound with a plural head, such as `layout rules of the widgets`, reads the same way.

`of` never gives a verb.
A plural noun that the rule turns into a verb counts as a verb for the nouns after it, so one clause gives one verb.

The alternative was to read every plural noun before a preposition as a verb.
On GUM, with a noun before and a preposition after, that is wrong 55 times in 64.

## Measures

All three were taken on the tags of the corrected tagger against the model alone.

| Measure | Old model | hakkiri model | Corrected |
|:--|--:|--:|--:|
| Universal Dependencies (UD) English GUM, 56,516 tokens | 90.26% | 92.16% | 92.53% |
| 200 judged conflicts of our documents, right | 131 | 103 | 140 |
| Alerts of `NounStack` and `ThingsSpeakNoun` removed, judged false | | | 68 of 68 |

The rules added no alert.
The alert counts come from the nine repositories that run Vale, at `--min-level suggestion`.
