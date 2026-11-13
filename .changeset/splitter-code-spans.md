---
hakkiri: patch
"verctl:group": fixed
---

The sentence splitter reads code spans: a code span begins a sentence after a full stop and ends one before a full stop.
A unit of measure after a number, as in `92 ms. Tests`, ends a sentence before a capital word.
`vs.` ends no sentence, and a date is no list number.
