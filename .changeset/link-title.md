---
hakkiri: patch
"verctl:group": added
---

A scope `title` selects the title of a Markdown link, image and link reference definition, which no other scope reads, as in Vale, where only `raw` does.
A rule that lists `title`, such as ProseStyle's `BareURL` for hakkiri, sees a URL in `[t](u "see https://x")`.
