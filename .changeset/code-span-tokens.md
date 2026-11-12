---
hakkiri: patch
"verctl:group": changed
---

A code span is a noun in a `sequence` rule: the tagger reads a noun in its place, its tag is `NN`, and a pattern reads it as the letter `X`.
A rule of a noun and a verb, such as `ThingsSpeakNoun`, flags `` `flag` says `` as it flags `the flag says`, and a rule for four lowercase nouns, such as `NounStack`, counts no code span.
The square brackets of an intra-doc link, as in ``[`Name`]``, belong to the span.
The tagger corrects the verbs of saying and asking after a noun: `the docs say` reads as a verb, `for the asks that wait` reads `asks` as a noun, and `the hook told of the file` reads `told` as a participle, so a rule with a verb tag on its verb token flags a thing that speaks and no request or participle.
