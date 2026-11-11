---
hakkiri: patch
"verctl:group": added
---

A key that names a rule and a level under a glob of `files` now runs that rule, though no glob lists its style, as a rule setting does in Vale.
The word `on` turns a rule on at the level of its rule file, as Vale's `YES` does.
An empty `styles` list also drops the rule settings before it.
