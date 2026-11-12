---
hakkiri: patch
"verctl:group": changed
---

The tagger corrects the model's tags where it is wrong on our prose.
The cases are a possessive `'s` after a noun, a modal written in capitals, a name in capitals and a semicolon.
A verb after a noun phrase, which the model reads as a plural noun, is the fifth.
On UD English GUM the tags match the gold tags 92.53% of the time, up from 92.16%.
`NounStack` and `ThingsSpeakNoun` give 68 fewer alerts on the nine repositories of the alert diff, all false.
A title is still tagged as the model tags it.
