---
hakkiri: minor
"verctl:group": changed
---

The globs that match a file now rank by specificity, not by the order written.
A more specific glob wins for the same key wherever it is written.

At equal specificity the project beats a package.
The top level of a config file ranks below every glob, so a package glob beats the project's top-level `code_span_size`.

The project can force a value with the YAML tag `!override`.
A forced value applies after every value without the tag.
A package's settings file that holds the tag is an error, and so is any other tag.
