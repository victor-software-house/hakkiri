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
4. **A preposition attaches the span to a prose noun.**
   In `` the notice for `cmd-7` says `` the span ends a phrase that `notice` heads, and the run starts there.
   The walk follows the chain, so `` the lock of the instance at `paths` says `` lands on `lock`.
   Only a token that reads one word does this, so `NounStack`, whose token repeats, never reads a code span as a noun.
   The alternative was to refuse every code span, which loses 8 true alerts of the nine repositories.
5. **A noun right before the span is no head.**
   `` the snapshot `request` asks for `` and `` the scenario `A` says `` have the same shape, and only the second is a name.
   Taking the noun before the span brings back 7 of the 8 lost alerts and 23 of the 64 false ones.
   Cues for a stranded preposition and a determiner before the span brought 6 of the 8 and 8 of the 64, so the change leaves them out.

## Measures

The alert diff of nine repositories, at `--min-level suggestion`, against the stack top before the change.
I judged by hand every alert it removes or adds.

| Rule | Removed | False | True | Added | True | False |
|:--|--:|--:|--:|--:|--:|--:|
| `NounStack` | 41 | 41 | 0 | 1 | 0 | 1 |
| `ThingsSpeakNoun` | 31 | 23 | 8 | 16 | 12 | 4 |

A removed alert is false when its noun is a code span, or the word `asks` read as a plural noun.
Of the 8 true ones that it removes, 5 come back on the prose noun, among 7 new true alerts.
The 3 that stay lost name a scenario that says twice, as in `` the scenario `A` says ``, and a check that agrees, as in `` the `UNIQUE` check on `session.dir` agrees ``.
The 4 false alerts that return are the plural noun `asks` in `` the `/compact` asks ``.
The false `NounStack` alert is not about a code span: a fix to the count of converted verbs restores it.
