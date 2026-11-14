## ADDED Requirements

### Requirement: The alt text of an inline img tag is an alt node

hakkiri SHALL give the alt text of an `img` tag in a line of Markdown an `alt` node of its own, as it gives one to the alt text of a Markdown image.
The paragraph SHALL read around the node, and a rule on `link` or on sentences SHALL not select it, as in Vale.

#### Scenario: An img tag alone and inside links

- **WHEN** a Markdown file holds the paragraphs `Build <a href="u"><img src="b.svg" alt="the badge"></a> now.`, `See <img src="l.png" alt="our logo"> here.` and `[link <img src="x" alt="a badge"> here](u) end.`
- **THEN** the `alt` scope reads `the badge`, `our logo` and `a badge`, with `badge` and `logo` at columns 45, 31 and 27, as Vale 66bbb85 places them
- **AND** no `link` block and no sentence holds those words
