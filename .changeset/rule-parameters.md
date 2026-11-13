---
hakkiri: patch
"verctl:group": added
---

A glob of `files` takes `params`, a mapping from a rule name to the fields of that rule it sets, such as `Vale.Repetition: {max: 1}`, as Vale's `Style.Rule[param] = value` does.
A parameter reaches the files of the glob that sets it, where Vale applies it to every file, and a later glob wins for the same parameter.
A parameter that cannot be set is an error that names the config line.
That covers a level, `tokens` and the other fields that hold lists or prose, and a field the rule's check does not define.
It also covers a value of the wrong kind, a rule that does not exist, and any parameter of `Vale.Terms` or `Vale.Avoid`.
