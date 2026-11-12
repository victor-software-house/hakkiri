# Design

## Decisions

1. **A separate scope named `title`.**
   Vale reads a link title in `raw` only, so hakkiri adds the one scope a rewritten rule needs and keeps `text`, `link` and `alt` as they are.
   The alternative was to put the title inside the `link` scope, which would make every `link` rule read it and differ from Vale.
2. **The node sits where its element sits.**
   A link's title is the last child of its `link` node, an image's is a child of its `image` node, which holds the alt text.
   A definition's is a node of its own, since a definition is a block.
   The alternative was a node over the whole element, which would repeat the label.
3. **The title is found from the end of the element.**
   markdown-rs gives the title text but no span, so the builder looks for the closing delimiter at the end of the element, and for the opening one before it that no backslash escapes, after white space.
   The alternative was to search for the title text, which finds an equal word in the label or the destination first.
4. **The text is the title as written.**
   Character references decode as in an `alt`, and a backslash escape stays, so a rule sees what the file holds.
5. **A title is selected by nesting, through `&`.**
   `title & link` selects the title of a link, as `sentence & paragraph` selects the sentences of paragraphs, and `title & image` that of an image.
   The element around a title is a name the title's parts may use beside its own, and an item still needs a part that names the block itself, so `link` alone selects no title.
   The alternative was a scope for each pair, such as `link.title`, which adds a name for each element that may hold one.
6. **An HTML `title` attribute is a title too.**
   The attribute of a tag in an HTML block is a `title` node, and a line of text reads it through `raw` only, as in Vale.
   The attribute of an `a` tag sits in an empty `link` node, and that of an `img` tag in an empty `image` node, so `title & link` and `title & image` select it as they select a Markdown title.

## Overrides

A rule names the scope as `scope: title`, or lists it with others, such as `[text, title]`.
No config key turns it on or off.
