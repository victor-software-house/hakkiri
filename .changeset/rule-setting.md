---
hakkiri: patch
"verctl:group": added
---

A key that names a rule and a level under a glob of `files` now runs that rule, though no glob lists its style, as a rule setting does in Vale.
An empty `styles` list also drops the rule settings before it.
