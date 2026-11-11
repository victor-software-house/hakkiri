---
hakkiri: patch
"verctl:group": added
---

The top-level string values of a Markdown or MDX file's front matter are linted as text, as in Vale.
YAML between `---` lines, TOML between `+++` lines and JSON between `;;;` lines are read, and a rule scoped `text`, `frontmatter` or `frontmatter.<key>` reads a value.
A block or token pattern leaves the front matter as written.
