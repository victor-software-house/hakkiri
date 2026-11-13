# Design

## Decisions

### 1. The model reads a noun in place of a code span

`Tagger::tag_code` tags the sentence with the word `name` where a code span stands, and gives the span the tag `NN`.
With the letter `x` the model tags a symbol after a determiner, as in `the x check`, so the rule of a noun and a verb missed the span in several of the places a noun stands.
`name` gets `NN` in each of the places measured: after a determiner, after `as`, after a preposition, between square brackets and before a noun.
The alternative was to refuse the span, which loses most of the true alerts of `ThingsSpeakNoun` (decision 5).

### 2. A pattern reads a code span as `X`

`ThingsSpeakNoun` reads a noun that matches `[A-Za-z][\w-]*` with no case.
`NounStack` reads a lowercase word that matches `[a-z][a-z-]*`.
The capital letter satisfies the first and fails the second, so the walk needs no rule for either.
`NounStack` says that each noun is a lowercase word, and its own case leaves a code span out of a stack.

### 3. The brackets of an intra-doc link belong to the span

In Rust comments ``[`Name`]`` is a link, and the tagger saw a closing bracket where a noun stood.
`with_link_brackets` joins the three tokens when no byte lies between them.

### 4. The labels decide what a rule flags

The written definition of each rule comes from its message, its comment and its cases in the package.
`ThingsSpeakNoun` flags a word of saying, telling, knowing, thinking, wanting, agreeing or asking whose subject is a thing, and a code name is a thing.
`NounStack` flags four lowercase common nouns in a row, and a code span is none of them.

### 5. The refusal and a head noun were measured

Refusing the span gives 44 of the 268 true alerts of `ThingsSpeakNoun` that the tools diverge on.
Letting a preposition attach the span to the noun before it gives 56, and a head noun with two cues gives 81.
Reading the span as a noun gives 264, and no rule about prepositions.

### 6. The verb token of `ThingsSpeakNoun` carries a tag, and the tagger corrections make the tag hold

Without a tag the rule flags a noun or a participle that has the shape of a verb.
Of its 26 false alerts, 20 read `asks` as a verb where it names a request, and 6 read `asked about` or `told of` as a past tense.
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

The alerts of nine repositories are those of Vale 3.26.0 and of hakkiri builds.
The builds are: before the refusal (`P0`), with it (`P1`), this change before the tag (`N`), the tag with the tagger of `N` (`Tag`), and this change (`Fix`).
Each alert that the tools do not all give was labelled true or false against the definitions, 336 of `ThingsSpeakNoun` and 558 of `NounStack`.
Two readers labelled each set, and the few that differed were settled by the definition.
The 415 `NounStack` alerts that no code span touches differed on 9, all a hyphenated word before three nouns, settled as false since a hyphenated word is an adjective.

The configuration of one repository named a style that the package dropped and mapped Rust files to Markdown, which hakkiri reads natively.
The run that produced `P0` to `Fix` converted it as it stood, so hakkiri gave that repository no alert, and 40 alerts of its comments came to light when the conversion was corrected.
Of those, 31 of 32 `ThingsSpeakNoun` alerts and 1 of 8 `NounStack` alerts are true.
The table counts them for `Fix` only; the older states ran before the correction and lack them.

| Rule | Tool | Gives | True | False | Vale's true missed |
|:--|:--|--:|--:|--:|--:|
| `ThingsSpeakNoun` | Vale | 22 | 21 | 1 | |
| | `P0` | 84 | 69 | 15 | 18 |
| | `P1` | 53 | 46 | 7 | 19 |
| | `N` | 293 | 266 | 27 | 4 |
| | `Tag` | 246 | 230 | 16 | 6 |
| | `Fix` | 313 | 305 | 8 | 0 |
| `NounStack` | Vale | 278 | 16 | 262 | |
| | `P0` | 251 | 30 | 221 | 16 |
| | `N` | 210 | 30 | 180 | 16 |
| | `Fix` | 240 | 44 | 196 | 14 |

`Fix` gives every true alert of Vale for `ThingsSpeakNoun`.
Of its 8 false alerts one is also Vale's, `tell failures apart`, a literal verb.
Three are `asked to end` twice and `asked online`, a thing that is asked and no thing that asks, which a tagger cannot read without knowing what asks.
Three are `asked for, or`, `asked about` and `told of`, a participle that the package's rule removes with a tag on the token that follows the verb.
One is `the consumer wants the error` in a scenario, where the two readers differed and the definition passes a person.
`NounStack` gives 196 false alerts where Vale gives 262, and 44 true where Vale gives 16.
The 14 true alerts of Vale that `Fix` lacks are the words that `Fix` flags, since Vale's span runs on through the target of a link, and none is a miss.
Of the 196 false alerts, 12 hold a hyphenated word before three nouns, which the rule's pattern accepts and the definition does not.
GUM is right on 52,298 of 56,516 tokens, 92.54%, as before the change, and the 200 judged conflicts count 134 right against 133.
