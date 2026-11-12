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


## Measures

The alerts of nine repositories at `--min-level suggestion` are those of Vale 3.26.0 and of hakkiri builds: before the refusal (`P0`), with it (`P1`), with the head noun (`B`), with a head noun and two cues (`A`), this change before the tag (`N`), the tag with the tagger of `N` (`Tag`), and this change (`Fix`).
Each alert that the tools do not all give was labelled true or false against the definitions, 304 of `ThingsSpeakNoun` and 540 of `NounStack`.
Two readers labelled the `ThingsSpeakNoun` alerts, and two of 296 differed before I settled them by the definition.
One reader labelled the 415 `NounStack` alerts that no code span touches, and two the other 116.
A third reader labelled the 18 alerts that only the later builds give, and differed on one, settled by the definition.

| Rule | Tool | Gives | True | False | Vale's true missed |
|:--|:--|--:|--:|--:|--:|
| `ThingsSpeakNoun` | Vale | 22 | 21 | 1 | |
| | `P0` | 84 | 69 | 15 | 18 |
| | `P1` | 53 | 46 | 7 | 19 |
| | `B` | 69 | 58 | 11 | 18 |
| | `N` | 293 | 266 | 27 | 4 |
| | `Tag` | 246 | 230 | 16 | 6 |
| | `Fix` | 278 | 274 | 4 | 0 |
| `NounStack` | Vale | 278 | 16 | 262 | |
| | `P0` | 251 | 39 | 212 | 16 |
| | `N` | 210 | 39 | 171 | 16 |
| | `Fix` | 220 | 43 | 177 | 16 |

`Fix` gives all the alerts of Vale that are true for `ThingsSpeakNoun`.
Of its 4 false alerts one is also Vale's, `tell failures apart`, a literal verb.
The other 3 are `asked to end` twice and `asked online`, a thing that is asked and no thing that asks, which a tagger cannot read without knowing what asks.
Of the 16 true alerts of `NounStack` that Vale gives and `Fix` does not, 9 are the same words that `Fix` flags, since Vale's span runs on through the target of a link, and the other 7 are words the tagger reads as a verb, an adverb or an adjective in a run of nouns, such as `split pull request`.
The 4 true alerts that `Fix` adds are `pull request` after two nouns.
GUM is right on 52,298 of 56,516 tokens, 92.54%, where the tagger before the change is right on 52,294, 92.53%.
Of the 200 judged conflicts, the scoring I could rebuild counts 133 right before the change and 133 after it, and the 140 of `tag-corrections` used a count I could not repeat.
