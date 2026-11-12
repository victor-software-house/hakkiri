---
hakkiri: patch
"verctl:group": added
---

A `sequence` rule takes `forms` in place of `tokens`: a list of mappings that each hold `tokens`.
The rule matches where any form does, and every alert carries the one rule name, so a check with several shapes, such as a noun and a verb or a pronoun and a verb, is one rule.
Two forms that match overlapping words give the match that starts first.
A rule that sets both `tokens` and `forms`, or neither, is an error, and a fault in a form names it, as `forms[1].tokens[0].pattern`.
A rule with one `tokens` list gives the alerts it gave before.
Vale stops with a panic on a rule with `forms` and no `tokens`.
