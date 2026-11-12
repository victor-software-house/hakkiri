---
hakkiri: patch
"verctl:group": changed
---

A fault in the config names the line of its own key, also for a glob that holds `.`, `*`, `/`, `[` or `{`, and for a config written in flow style.
A fault of the first glob that sets a rule used to name the line of a later glob that sets it.
