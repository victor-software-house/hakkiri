---
hakkiri: patch
"verctl:group": fixed
---

A `code` rule reads an MDX file's `{…}` expressions and its JSX elements with no children, as in Vale.
The tags of an element with children and a `{/* … */}` comment still read as no node.
