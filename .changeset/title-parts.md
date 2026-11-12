---
hakkiri: patch
"verctl:group": added
---

`title & link` and `title & image` select the title of a link and of an image, and a new scope `image` selects the alt text of an image.
The `title` attribute of an HTML tag, in a block or in a line of text, and of a JSX element is a title too, and `title & link` and `title & image` select that of an `a` and an `img` tag.
The title of a tag in the text of a link belongs to that tag, so `title & link` leaves it out.
`title & ~link` and `title & ~image` leave out the titles of links and of images.
