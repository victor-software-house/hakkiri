---
hakkiri: patch
"verctl:group": added
---

Math in a Markdown or MDX file reads as code, as in Vale.
An inline span such as `$x$` follows Pandoc's delimiter rules, so `$5 and $10` stays prose, and a block that starts with `$$` ends at a line that ends with `$$`.
No text rule reports math, and a `code` rule reads it.
