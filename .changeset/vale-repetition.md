---
hakkiri: patch
"verctl:group": added
---

The built-in style `Vale` always exists and holds the rule `Vale.Repetition`, with Vale's defaults, so `styles: [Vale]` is no longer refused when no vocabulary holds a term, and it reports a word that repeats, such as `the the`.
Accepted terms hold a repeat back, and a level, `on` and `off` set by the name apply to it.
Two words that differ in case compare character by character, as in Vale, so `ΣΑΣ` and `σασ` are a repeat.
