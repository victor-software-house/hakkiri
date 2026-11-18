# Design

## Decisions

1. **A separate scope named `title`.**
   Vale reads a link title in `raw` only.
   So hakkiri adds the one scope a rewritten rule needs, and keeps `text`, `link` and `alt` as they are.
   - The alternative was to put the title inside the `link` scope.
     Every `link` rule would then read it and differ from Vale.
2. **The node sits where its element sits.**
   A link's title is the last child of its `link` node.
   An image's title is a child of its `image` node, which also contains the alt text.
   A definition's title is a node of its own, since a definition is a block.
   - The alternative was a node over the whole element, which would repeat the label.
3. **The title is found from the end of the element.**
   markdown-rs gives the title text but no span.
   So the builder looks for the closing delimiter at the end of the element.
   Then it looks before it for the opening one, after white space, that no backslash escapes.
   - The alternative was to search for the title text, which finds an equal word in the label or the destination first.
4. **The text is the title as written.**
   Character references decode as in an `alt`, and a backslash escape stays, so a rule sees what the file contains.
5. **A title is selected by nesting, through `&`.**
   `title & link` selects the title of a link, as `sentence & paragraph` selects the sentences of paragraphs.
   `title & image` selects that of an image.
   - The title's parts may use the element around a title as a name beside their own.
     An item still needs a part for the block itself, so `link` alone selects no title.
   - The alternative was a scope for each pair, such as `link.title`, which adds a name for each element that may have one.
6. **An HTML `title` attribute is a title too.**
   The attribute of an HTML tag is a `title` node, in a block and in a line of text.
   This also applies to a JavaScript XML (JSX) element in MDX, through its `title` attribute.
   - Vale reads it through `raw` only, so a rule that `raw` carries for titles moves onto `title` with no loss.
   - The attribute of an `a` tag sits in an empty `link` node, and that of an `img` tag in an empty `image` node.
     So `title & link` and `title & image` select it as they select a Markdown title.

## Overrides

A rule sets the scope as `scope: title`, or lists it with others, such as `[text, title]`.
No config key turns it on or off.
