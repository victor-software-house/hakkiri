---
hakkiri: patch
"verctl:group": added
---

A vocabulary builds two rules, as in Vale: `Vale.Terms` reports an accepted term written in another case, and `Vale.Avoid` reports a rejected term.
List the style `Vale` in `styles` to run them, or set them by name.
A vocabulary that a glob names builds `Vale.<Name>.Terms` and `Vale.<Name>.Avoid`, which run on that glob's files.
