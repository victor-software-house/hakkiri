# Design

## Decisions

1. **The model reads a noun in place of a code span.**
   `Tagger::tag_code` tags the sentence with the word `name` where a code span stands, and gives the span the tag `NN`.
   With the letter `x` the model tags a symbol after a determiner, as in `the x check`, so the rule of a noun and a verb missed the span in several of the places a noun stands.
   `name` gets `NN` in each of the places measured: after a determiner, after `as`, after a preposition, between square brackets and before a noun.
   The alternative was to refuse the span, which loses most of the true alerts of `ThingsSpeakNoun` (decision 5).
2. **A pattern reads a code span as `X`.**
   `ThingsSpeakNoun` reads a noun that matches `[A-Za-z][\w-]*` with no case, and `NounStack` reads a lowercase word that matches `[a-z][a-z-]*`.
   The capital letter satisfies the first and fails the second, so the walk needs no rule for either.
   `NounStack` says that each noun is a lowercase word, and its own case leaves a code span out of a stack.
3. **The brackets of an intra-doc link belong to the span.**
   In Rust comments ``[`Name`]`` is a link, and the tagger saw a closing bracket where a noun stood.
   `with_link_brackets` joins the three tokens when no byte lies between them.
4. **The labels decide what a rule flags.**
   The written definition of each rule comes from its message, its comment and its cases in the package.
   `ThingsSpeakNoun` flags a word of saying, telling, knowing, thinking, wanting, agreeing or asking whose subject is a thing, and a code name is a thing.
   `NounStack` flags four lowercase common nouns in a row, and a code span is none of them.
5. **The refusal and a head noun were measured.**
   Refusing the span gives 44 of the 268 true alerts of `ThingsSpeakNoun` that the tools disagree on.
   Letting a preposition attach the span to the noun before it gives 56, and a head noun with two cues gives 81.
   Reading the span as a noun gives 264, and no rule about prepositions.
6. **The verb token of `ThingsSpeakNoun` carries a tag, and the tagger corrections make it hold.**
   Without a tag the rule flags a noun or a participle that has the shape of a verb: 20 of the 26 false alerts read `asks` as a verb where it names a request, and 6 read `asked about` or `told of` as a past tense.
   The package takes `tag: ^(?:VB|VBD|VBP|VBZ)$` on that token.
   The pattern is anchored because a tag is a search, and `VB` alone also matches `VBN` and `VBG`.
   The tag alone cuts 34 true alerts and 4 of Vale's, since the model reads a verb after a noun as a noun, so the tagger gets these corrections, each reading the words and tags of one sentence:
   - The forms of `say`, `tell`, `know`, `think`, `believe`, `understand`, `insist` and `agree` are no nouns, and each is a verb at every use in GUM.
     A noun, adjective or interjection tag on one is a verb, unless a determiner, a number, a possessive or a preposition comes before it.
   - The word before such a verb is its subject: a `VBZ` after a noun is a plural noun (`the CI builds agree`), and an adjective after a determiner is a noun (`the report agree`).
   - A base form after a possessive is a noun, and `own` is an adjective (`the client's resolver asks`).
   - A lowercase first word before `say` or `ask` is a name (`cargo asks again`).
   - A plural noun that starts a sentence before a determiner is a verb, except before `that` (`Takes every waiting ask`).
   - `ask` and `asks` are a noun before a relative clause, a possessive ending or `and` and a determiner, and after a preposition, a gerund or an imperative that opens a noun phrase.
     They are a verb before a preposition with no object, and a noun after a noun in a clause of its own.
   - `asked` and `told` are a participle in the noun phrase of a preposition (`the hook told of the file`) and after `one` before a preposition with no object, and a past tense after a noun in any other place (`a batch a request asked for`).
   - `pull` before `request` is a noun.
   A rule that read any verb in a noun phrase as a noun lost 78 tokens of GUM, and a rule for the middle of four nouns lost 2, so both were dropped.

7. **A pattern check reads a code span as one `*` and drops a match only at its edges.**
   `run_existence` searched the text where a code span is the letter `x`, and dropped each match that overlapped a span.
   Vale masks a span as a run of `*`, and a match that runs across one is a match: 452 of the 487 alerts of `ClauseChain` that Vale gave and hakkiri did not held a span inside the match, and 39 of 50 sampled were real chains.
   `Text::pattern_reading` now gives the text with each span as one `*`, a byte that no word and no space matches and that the text already uses for masked markup, so every offset holds.
   `Text::edge_in_code` drops a match whose first or last byte lies in a span, and keeps one with text on both sides.
   The sequence walk reads tokens from the tagger and keeps decision 2, so the two readings do not meet.
   `substitution`, `occurrence`, `repetition` and the rest keep the refusal of an overlap.
8. **The tagger reads the nouns that technical prose uses where the model reads another part of speech.**
   The model reads `split`, `round`, `check`, `prompt`, `fade` and `thinking` as one other tag in `the host load split pull request`, `the mask round pull request`, `the peer user id check`, `the prompt template pull request`, `the fade removal pull request` and `the thinking request fields decision`, so a run of four nouns lost one.
   The model lists some words with one tag and never scores them, and for `prompt` the score of a noun is 25 points under an adjective, so no margin of the model separates `prompt` from `final`.
   `nouns_of_technical_prose` holds the six words, and reads one as a noun after a determiner or a possessive, after an adjective unless it is a participle, in the title of a pull request after a noun, and as the base form after two singular nouns before a preposition or a stop.
   A model margin, a rule for a word between nouns and a rule for any adjective before three nouns were measured and dropped: each cost GUM tokens or gave false alerts such as `record then places` and `outbound mask speed pull request`.
   A rule that read a hyphenated word as an adjective, which the definition of `NounStack` holds, lost 6 tokens of GUM and was dropped; the rule's pattern accepts a hyphen, and 12 alerts of `NounStack` that are false by the definition rest on it.

## Measures

The alerts of nine repositories are those of Vale 3.26.0 and of hakkiri builds: before the refusal (`P0`), with it (`P1`), with the head noun (`B`), with a head noun and two cues (`A`), this change before the tag (`N`), the tag with the tagger of `N` (`Tag`), and this change (`Fix`).
Each alert that the tools do not all give was labelled true or false against the definitions, 304 of `ThingsSpeakNoun` and 550 of `NounStack`.
Two readers labelled the `ThingsSpeakNoun` alerts, and two of 296 differed before I settled them by the definition.
Two readers labelled the 415 `NounStack` alerts that no code span touches, and differed on 9, all a hyphenated word before three nouns, settled as false since a hyphenated word is an adjective.
Two readers labelled the other 115, and two more the 10 alerts that only `Fix` gives, with no difference.

| Rule | Tool | Gives | True | False | Vale's true missed |
|:--|:--|--:|--:|--:|--:|
| `ThingsSpeakNoun` | Vale | 22 | 21 | 1 | |
| | `P0` | 84 | 69 | 15 | 18 |
| | `P1` | 53 | 46 | 7 | 19 |
| | `B` | 69 | 58 | 11 | 18 |
| | `A` | 97 | 83 | 14 | 17 |
| | `N` | 293 | 266 | 27 | 4 |
| | `Tag` | 246 | 230 | 16 | 6 |
| | `Fix` | 281 | 274 | 7 | 0 |
| `NounStack` | Vale | 278 | 16 | 262 | |
| | `P0` | 251 | 30 | 221 | 16 |
| | `N` | 210 | 30 | 180 | 16 |
| | `Fix` | 232 | 43 | 189 | 14 |

`Fix` gives every true alert of Vale for `ThingsSpeakNoun`.
Of its 7 false alerts one is also Vale's, `tell failures apart`, a literal verb.
Three are `asked to end` twice and `asked online`, a thing that is asked and no thing that asks, which a tagger cannot read without knowing what asks.
Three are `asked for, or`, `asked about` and `told of`, a participle that the package's rule removes with a tag on the token that follows the verb.
`NounStack` gives 189 false alerts where Vale gives 262, and 43 true where Vale gives 16.
The 14 true alerts of Vale that `Fix` lacks are the words that `Fix` flags, since Vale's span runs on through the target of a link, and none is a miss.
Of the 189 false alerts, 12 hold a hyphenated word before three nouns, which the rule's pattern accepts and the definition does not.
The 7 true alerts of Vale that `N` lacked name `peer user id check`, `mask round pull request`, `host load split pull request`, `load split pull request`, `fade removal pull request`, `prompt template pull request` and `thinking request fields decision`, and `Fix` gives all of them.
GUM is right on 52,298 of 56,516 tokens, 92.54%, as before the change, and the 200 judged conflicts count 134 right against 133.

On the same nine repositories the package's `ClauseChain` gave 1,020 alerts before decision 7 and 1,523 after it, where Vale gives 1,492.
The match text differs on 371 of them, where Vale prints the mask as `*` and hakkiri prints `x`, so the diff counts each as changed.
Two readers labelled the 210 alerts that only one tool gives or that move, and differed on 9, settled by the rule that each part of a chain holds a verb.
Pairing the rest by line leaves 11 true alerts of Vale that hakkiri lacks, the same 11 as before the change, in a comment block or a list item that the sentence splitter reads another way.
It leaves 26 true alerts that only hakkiri gives and 18 false ones, 9 of them already given before decision 7.
The 9 new false alerts are a semicolon between two noun phrases or two code spans, in a comment or a list.
`ThingsSpeakNoun` gives the same alerts as before, `PrepositionChain` gives one more of Vale's, and `NounStack` gives 12 more.
