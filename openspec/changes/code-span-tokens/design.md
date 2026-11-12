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

## Measures

The alerts of nine repositories at `--min-level suggestion` are those of Vale 3.26.0 and of five hakkiri builds: before the refusal (`P0`), with it (`P1`), with the head noun (`B`), with a head noun and two cues (`A`), and this change (`N`).
Each alert that the six do not all give was labelled true or false against the definitions, 296 of `ThingsSpeakNoun` and 531 of `NounStack`.
Two readers labelled the `ThingsSpeakNoun` alerts, and two of 296 differed before I settled them by the definition.
One reader labelled the 415 `NounStack` alerts that no code span touches, and two the other 116.

| Rule | Tool | Gives | True | False | Vale's true missed |
|:--|:--|--:|--:|--:|--:|
| `ThingsSpeakNoun` | Vale | 19 | 19 | 0 | |
| | `P0` | 81 | 67 | 14 | 18 |
| | `P1` | 50 | 44 | 6 | 19 |
| | `B` | 66 | 56 | 10 | 18 |
| | `N` | 290 | 264 | 26 | 4 |
| `NounStack` | Vale | 278 | 16 | 262 | |
| | `P0` | 251 | 39 | 212 | 16 |
| | `P1` | 211 | 39 | 172 | 16 |
| | `N` | 210 | 39 | 171 | 16 |

The 4 true alerts of Vale that `N` misses are tagger errors on words of prose.
The 16 of `NounStack` come from a link that Vale reads across its text and hakkiri reads inside its own.
Of the 26 false alerts of `N`, 20 read `ask` or `asks` as a verb where it is a noun, 23 of the 26 in one repository that uses the noun `ask`, and 6 read a participle such as `asked about` as a verb.
The rule has no tag on its verb token, so only a package change can remove them.
`NounStack` gives 41 fewer false alerts than `P0` for a code span in a stack, and one false alert remains that has no code span.
