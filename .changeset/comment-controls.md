---
hakkiri: patch
"verctl:group": added
---

An HTML comment in a Markdown file, and a `{/* … */}` comment in an MDX file, turns rules off and on for the text after it, with the reach of Vale's comments.
Write `<!-- hakkiri off -->` and `<!-- hakkiri on -->`, or `<!-- hakkiri Style.Rule: off -->` and `<!-- hakkiri Style.Rule: on -->` for one rule.
`<!-- hakkiri Style.Rule: suggestion -->`, `warning` or `error` sets the level of a rule's alerts from there on.
Vale's own comments, such as `<!-- vale off -->` and `<!-- vale Style.Rule = NO -->`, work as they do in Vale.
