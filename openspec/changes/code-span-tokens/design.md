# Design

## Decisions

1. **The walk refuses the word.**
   Each word of a sentence carries whether it reads a code span, and `accepts` returns false for it when the token is not negated.
   The alternative was to drop any match that overlaps a code span, as the other checks do, which also drops a match that passes a code span through a negated token or a `skip`.
2. **A negated token holds at a code span.**
   A negated token names what a word is not, and a code span is neither the tag nor the text it names.
   The alternative was to refuse the word for every token, which makes `files` that no noun follows fail at `` files `x` ``.
3. **The tags stay as they are.**
   The tagger still tags the `x` of a code span, so the tags of the words around it are the ones the model gives, as Vale's are.
   The alternative was to leave the span out of the tagged text, which moves the words next to it.
4. **A prose noun beside a code span is not recovered.**
   In `` the scenario `A` says `` the noun that speaks is `scenario`, and the span sits between it and the verb, so no run holds them.
   Alerts of that shape fall away with the code names.
   The alternative was to let a walk step over a code span, which changes what every `skip` and `min` count.

## Measures

The alert diff of nine repositories, at `--min-level suggestion`, against the stack top before the change.

| Rule | Alerts removed | False | True violation lost |
|:--|--:|--:|--:|
| `NounStack` | 41 | 41 | 0 |
| `ThingsSpeakNoun` | 31 | 23 | 8 |

A removed alert is false when its noun is a code span, or the word `asks` read as a plural noun.
The 8 lost ones name a prose noun that speaks, with a code span between it and the verb, as in `` the notice for `cmd-7` says ``.
