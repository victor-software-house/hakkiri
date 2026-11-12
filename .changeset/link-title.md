---
hakkiri: patch
"verctl:group": added
---

A scope `title` selects the title of a Markdown link, image and link reference definition, which no other scope reads, as in Vale, where only `raw` does.
A rule that lists `title`, such as ProseStyle's `BareURL` for hakkiri, sees a URL in `[t](u "see https://x")`.
`title & link` and `title & image` select the title of a link and of an image, and a new scope `image` selects the alt text of an image.
The `title` attribute of a tag in an HTML block is a title too, and `title & link` and `title & image` select that of an `a` and an `img` tag.
`title & ~link` and `title & ~image` leave out the titles of links and of images.
