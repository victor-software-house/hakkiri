# Spec Delta

## ADDED Requirements

### Requirement: A node records its first-child place

The tree of a Markdown or MDX file SHALL record on each node whether it is the first child of the list item, quote or table row that holds it.  
The parser decides, from every child of that container.

#### Scenario: A paragraph after a fenced block in an item

- **WHEN** a Markdown file holds a list item whose first child is a fenced block and whose second child is the paragraph `After the fence.`
- **THEN** the tree holds a `paragraph` node for `After the fence.` that is not first in its item

#### Scenario: A paragraph that opens an item

- **WHEN** a Markdown file holds a list item whose first child is the paragraph `Opens the item.`
- **THEN** the tree holds a `paragraph` node for `Opens the item.` that is first in its item

### Requirement: A block takes the first place as Vale reads it

A fenced block, a table, an empty item and an HTML block SHALL take the first place when the HTML has an end tag that is not inline, or text outside its tags and comments.  
A thematic break, a link definition and any other HTML block SHALL leave it free, as in Vale 66bbb85, which reads HTML with the tokenizer of golang.org/x/net/html.

#### Scenario: A paragraph after a thematic break in a quote

- **WHEN** a Markdown file holds a quote whose first child is a thematic break and whose second child is the paragraph `After the rule.`
- **THEN** the tree holds a `paragraph` node for `After the rule.` that is first in its quote

#### Scenario: A paragraph after a comment and a div in a quote

- **WHEN** a Markdown file holds a quote whose first child is the HTML block `<!-- c --> <div>x</div>` and whose second child is the paragraph `After the div.`
- **THEN** the tree holds a `paragraph` node for `After the div.` that is not first in its quote

#### Scenario: A paragraph after a bare start tag in a quote

- **WHEN** a Markdown file holds a quote whose first child is the HTML block `<div>` and whose second child is the paragraph `After the tag.`
- **THEN** the tree holds a `paragraph` node for `After the tag.` that is first in its quote

#### Scenario: A paragraph after a processing instruction with a tag in it

- **WHEN** a Markdown file holds a quote whose first child is the HTML block `<?php echo "<div>"; ?>` and whose second child is the paragraph `After the block.`
- **THEN** the tree holds a `paragraph` node for `After the block.` that is not first in its quote
