---
hakkiri: patch
"verctl:group": fixed
---

A YAML vocabulary term with `ignorecase: true` is quoted whole, so the period of `Node.js` matches a period and not any character.
Vale reads that period as a wildcard, and the `vale-differences` spec records the difference.
