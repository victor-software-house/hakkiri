## MODIFIED Requirements

### Requirement: Markdown nodes carry Vale scope names

hakkiri SHALL give each Markdown node the scope the design's table names.  
The table covers paragraphs, headings, block quotes, list items, table rows and cells, links, code spans, strong and emphasis text, and image alt text.  
The text of an HTML block adds table and figure captions, the content of a `data` element as `meta`, and the description of a page.

#### Scenario: A nested list

- **WHEN** a Markdown file holds `- One` and, indented under it, `- Two`
- **THEN** the tree holds a `list` node for `One` whose children include a `list` node for `Two`

#### Scenario: A table

- **WHEN** a Markdown file holds a table with header `| Name | Note |` and one body row `| a | b |`
- **THEN** the tree holds two `table.row` nodes, the first with two `table.header` cells and the second with two `table.cell` cells, each span covering its cell text

### Requirement: Code and markup are not prose

Fenced code, the tags of raw HTML, link definitions and front matter SHALL carry no prose scope, and SHALL appear in the tree only under `raw`.  
Inside a prose node, the node-segments spec makes such syntax a gap in the text.  
The text inside an HTML block is prose, and the html-block-text spec states which text and under which scope.

#### Scenario: A URL in a fenced block

- **WHEN** a Markdown file holds a fenced block with `curl https://example.com`
- **THEN** no prose node holds `https://example.com`, and the `raw` node of the file does

#### Scenario: A tag and its text in an HTML block

- **WHEN** a Markdown file holds an HTML block `<div class="note">Read this.</div>`
- **THEN** the tree holds a `paragraph` node for `Read this.` and no prose node for `<div class="note">`
