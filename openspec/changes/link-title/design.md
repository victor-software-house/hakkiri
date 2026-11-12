# Design

## Decisions

1. **A separate scope named `title`.**
   Vale reads a link title in `raw` only, so hakkiri adds the one scope a rewritten rule needs and keeps `text`, `link` and `alt` as they are.
   The alternative was to put the title inside the `link` scope, which would make every `link` rule read it and differ from Vale.
2. **The node sits where its element sits.**
   A link's title is the last child of its `link` node, an image's follows its `alt` node, and a definition's is a node of its own, since a definition is a block.
   The alternative was a node over the whole element, which would repeat the label.
3. **The title is found from the end of the element.**
   markdown-rs gives the title text but no span, so the builder looks for the closing delimiter at the end of the element, and for the opening one before it that no backslash escapes, after white space.
   The alternative was to search for the title text, which finds an equal word in the label or the destination first.
4. **The text is the title as written.**
   Character references decode as in an `alt`, and a backslash escape stays, so a rule sees what the file holds.

## Overrides

A rule names the scope as `scope: title`, or lists it with others, such as `[text, title]`.
No config key turns it on or off.
