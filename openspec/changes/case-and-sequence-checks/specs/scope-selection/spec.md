# Spec Delta

## MODIFIED Requirements

### Requirement: sentence selects each sentence

The scope `sentence` SHALL select each sentence of a paragraph, a heading or a table cell.  
A sentence SHALL have the scope of the block it is in, so a part such as `list` selects the sentences of a list item, and an exclusion such as `~blockquote` leaves out the sentences of a quote.

#### Scenario: Two sentences measured apart

- **WHEN** a rule has `extends: metric`, `message: "%s words"`, `scope: sentence`, `formula: words` and `condition: '> 3'`, and the input is `One two. Three four five six.`
- **THEN** hakkiri reports one alert, with the message `4.00 words`

#### Scenario: A sentence in a list item

- **WHEN** a rule has `scope: sentence & list` and the input holds the paragraph `One two.` and the list item `- Three four.`
- **THEN** hakkiri selects one sentence, `Three four.`

### Requirement: paragraph leaves out list items and quotes

The scope `paragraph` SHALL select a paragraph that opens no list item or block quote, and every later paragraph of an item or a quote, as Vale 3.26.0 does.

#### Scenario: A paragraph and a list item

- **WHEN** a rule has `scope: paragraph` and `tokens: [marker]`, and the input holds `- A marker item.` and a paragraph `A marker.`
- **THEN** hakkiri reports one alert, in the paragraph

#### Scenario: A later paragraph of a list item

- **WHEN** the same rule reads the list item `- First.` followed by the paragraph `A marker here.`, indented under it
- **THEN** hakkiri reports one alert, in the later paragraph

## ADDED Requirements

### Requirement: A block of text has the scope of one element

A block of text SHALL have the scope of one element, as in Vale 3.26.0.  
That element SHALL be the outermost list item, block quote, heading or table cell that the block is the first thing inside.  
A block that is the first thing inside none of them SHALL have its own scope: a paragraph, a heading or a table cell.  
A list item and a block quote SHALL be read through the blocks inside them.

#### Scenario: A heading in a quote

- **WHEN** a rule has `scope: heading` and `tokens: [marker]`, and the input is the quote `> A quote.` followed by `> # A marker heading`
- **THEN** hakkiri reports one alert, in the heading

#### Scenario: A list item in a quote

- **WHEN** a rule has `scope: list` and `tokens: [marker]`, and the input is the quote `> A quote.` followed by the list item `> - A marker item.`
- **THEN** hakkiri reports one alert, in the item

#### Scenario: A quote that opens an item

- **WHEN** a rule has `scope: list`, and the input is the list item `- > Opens the item.`
- **THEN** hakkiri selects one block, `Opens the item.`

#### Scenario: A paragraph after a fenced block in an item

- **WHEN** a rule has `scope: paragraph`, and the input is a list item that starts with a fenced block and holds the paragraph `After the fence.`
- **THEN** hakkiri selects one block, `After the fence.`

#### Scenario: A paragraph after a fenced block in a quote

- **WHEN** a rule has `scope: paragraph`, and the input is a quote that starts with a fenced block and holds the paragraph `> After the fence.`
- **THEN** hakkiri selects one block, `After the fence.`

#### Scenario: A paragraph after the first in a quote inside an item

- **WHEN** a rule has `scope: paragraph`, and the input is a list item that holds a quote of the paragraphs `Para one.` and `Para two.`
- **THEN** hakkiri selects one block, `Para two.`, and a rule with `scope: blockquote` selects `Para one.`

### Requirement: A dotted sentence is a sentence in a scope

A scope part written `sentence.` and a name SHALL read as that name joined to `sentence` with `&`, as in Vale 3.26.0.

#### Scenario: sentence.list

- **WHEN** a rule has `scope: sentence.list`, and the input is the paragraph `A paragraph.` and the list item `- An item.`
- **THEN** hakkiri selects one sentence, `An item.`

### Requirement: table selects cells

The scope `table` SHALL select the header and body cells of a table, and not its rows.  
The scope `table.row` SHALL select the rows.

#### Scenario: Cells and rows

- **WHEN** a rule has `scope: table`, and the input is a table of two columns with a header row and one body row
- **THEN** hakkiri selects four blocks, one for each cell
