# Spec Delta

## ADDED Requirements

### Requirement: A node records its first-child place

The tree of a Markdown or MDX file SHALL record on each node whether it is the first child of the list item, quote or table row that holds it.  
The parser decides, from every child of that container.  
A fenced block, a table, an empty item, or an HTML block with a tag that is neither void nor inline, before the node, means it is not first.  
A thematic break, a link definition, or an HTML block of only comments, void tags and inline tags, does not, as in Vale 66bbb85.

#### Scenario: A paragraph after a fenced block in an item

- **WHEN** a Markdown file holds a list item whose first child is a fenced block and whose second child is the paragraph `After the fence.`
- **THEN** the tree holds a `paragraph` node for `After the fence.` that is not first in its item

#### Scenario: A paragraph that opens an item

- **WHEN** a Markdown file holds a list item whose first child is the paragraph `Opens the item.`
- **THEN** the tree holds a `paragraph` node for `Opens the item.` that is first in its item

#### Scenario: A paragraph after a thematic break in a quote

- **WHEN** a Markdown file holds a quote whose first child is a thematic break and whose second child is the paragraph `After the rule.`
- **THEN** the tree holds a `paragraph` node for `After the rule.` that is first in its quote

#### Scenario: A paragraph after a comment and a div in a quote

- **WHEN** a Markdown file holds a quote whose first child is the HTML block `<!-- c --> <div>x</div>` and whose second child is the paragraph `After the div.`
- **THEN** the tree holds a `paragraph` node for `After the div.` that is not first in its quote
