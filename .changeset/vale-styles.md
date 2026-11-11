---
hakkiri: patch
"verctl:group": added
---

A Vale style loads as it is: a rule whose `extends` names another rule starts from it, with `key+` and `key-`, as in Vale.
A rule hakkiri cannot run, a `script` rule or a `doc()` scope, no longer stops its style from loading.
The rest of the style runs, and the run exits 2 for that rule until the config sets it `off`.
