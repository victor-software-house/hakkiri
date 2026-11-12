---
hakkiri: patch
"verctl:group": changed
---

A code span is a noun in a `sequence` rule: the tagger reads a noun in its place, its tag is `NN`, and a pattern reads it as the letter `X`.
A rule of a noun and a verb, such as `ThingsSpeakNoun`, flags `` `flag` says `` as it flags `the flag says`, and a rule for four lowercase nouns, such as `NounStack`, counts no code span.
The square brackets of an intra-doc link, as in ``[`Name`]``, belong to the span.
