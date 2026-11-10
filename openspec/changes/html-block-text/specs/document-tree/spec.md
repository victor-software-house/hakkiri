## MODIFIED Requirements

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
