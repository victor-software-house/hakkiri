## MODIFIED Requirements

### Requirement: Segments join into the text of a node

The text of a node SHALL be its segments in order.  
A line feed SHALL separate two segments with a line break between them.  
One space SHALL separate two cells of a table row.  
Any other gap between segments SHALL read as nothing.  
A segment MAY carry a reading that replaces what the gap gives: nothing, one space or one line feed before the segment, and `*` for each character of the segment.  
The text of an HTML tag and the text around it SHALL read as the html-block-text spec defines.

#### Scenario: A table row

- **WHEN** a Markdown file holds a table with the row `| a | b |`
- **THEN** the `table.row` node has the segments of its two cells, and its text is `a b`

#### Scenario: A tag between words

- **WHEN** a Markdown file holds `a<br>marker<br/>b`
- **THEN** the `paragraph` node has three segments, and its text is `a marker b`
