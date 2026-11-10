# scope-selection Specification

## Purpose

Selection decides which nodes of the document tree a rule reads, and what text, sentences and words a check sees in each of them.  
The [rule-files spec][rule-files] names the scopes; this spec fixes how they match.

## Requirements

### Requirement: A scope name matches by dotted prefix

A scope part SHALL select every node whose scope name equals it or starts with it followed by a dot.

#### Scenario: heading selects every level

- **WHEN** a rule has `scope: heading` and `tokens: [marker]`, and the input holds `# A marker` and `### Another marker`
- **THEN** hakkiri reports two alerts, one in each heading

#### Scenario: text.comment selects every comment

- **WHEN** a rule has `scope: text.comment` and `tokens: [marker]`, and a Rust file holds `// A marker.` and `/// Another marker.`
- **THEN** hakkiri reports two alerts

### Requirement: text selects the prose leaves

The scope `text` SHALL select each paragraph, heading and table cell, in a Markdown file and inside a comment.  
A list item or a block quote SHALL be read through the paragraphs inside it, so its text is read once.

#### Scenario: One alert per place

- **WHEN** a rule has `scope: text` and `tokens: [marker]`, and the input holds `# A marker`, a paragraph `A marker.` and a list item `- A marker`
- **THEN** hakkiri reports three alerts, one on each line

### Requirement: An unset scope reads as text

A rule of any check type except `metric` that sets no `scope` SHALL select what `text` selects.

#### Scenario: A table cell with no scope

- **WHEN** a rule has `tokens: [marker]` and no `scope`, and a table body cell holds `marker`
- **THEN** hakkiri reports one alert in that cell

### Requirement: paragraph leaves out list items and quotes

The scope `paragraph` SHALL select a paragraph that opens no list item or block quote, and every later paragraph of an item or a quote, as Vale 3.26.0 does.

#### Scenario: A paragraph and a list item

- **WHEN** a rule has `scope: paragraph` and `tokens: [marker]`, and the input holds `- A marker item.` and a paragraph `A marker.`
- **THEN** hakkiri reports one alert, in the paragraph

#### Scenario: A later paragraph of a list item

- **WHEN** the same rule reads the list item `- First.` followed by the paragraph `A marker here.`, indented under it
- **THEN** hakkiri reports one alert, in the later paragraph

### Requirement: sentence selects each sentence

The scope `sentence` SHALL select each sentence of a paragraph, a heading or a table cell.  
A sentence SHALL have the scope of the block it is in, so a part such as `list` selects the sentences of a list item, and an exclusion such as `~blockquote` leaves out the sentences of a quote.

#### Scenario: Two sentences measured apart

- **WHEN** a rule has `extends: metric`, `message: "%s words"`, `scope: sentence`, `formula: words` and `condition: '> 3'`, and the input is `One two. Three four five six.`
- **THEN** hakkiri reports one alert, with the message `4.00 words`

#### Scenario: A sentence in a list item

- **WHEN** a rule has `scope: sentence & list` and the input holds the paragraph `One two.` and the list item `- Three four.`
- **THEN** hakkiri selects one sentence, `Three four.`

### Requirement: A file extension narrows a scope

A last scope part that names a file extension, such as `rs` in `raw.rs`, SHALL limit the scope to files with that extension.
A negated part with an extension, such as `~heading.rs`, SHALL leave out nodes only in files with that extension.  
A scope item whose every part names another extension SHALL select nothing.

#### Scenario: raw.rs in two files

- **WHEN** a rule has `scope: raw.rs` and `tokens: [marker]`, and a Markdown file holds `A marker.` and a Rust file holds `// A marker.`
- **THEN** hakkiri reports one alert, in the Rust file

#### Scenario: A negated part for another language

- **WHEN** a rule has `scope: text & ~heading.rs` and `tokens: [marker]`, and a Markdown file holds `# A marker`
- **THEN** hakkiri reports one alert, on the heading

#### Scenario: An item that names only another language

- **WHEN** a rule has `scope: [~heading.rs]` and `tokens: [marker]`, and a Markdown file holds `# A marker` and `A marker.`
- **THEN** hakkiri reports no alert

### Requirement: An inline exclusion blanks the element

A part `~link`, `~code`, `~strong` or `~emphasis` SHALL leave the selected node in place and read the text of that inline element as spaces.
The exclusion SHALL apply only to the blocks that its own scope item selects.

#### Scenario: A word outside and inside a link

- **WHEN** a rule has `scope: text & ~link` and `tokens: [marker]`, and the input is `See marker and [marker](https://x.example).`
- **THEN** hakkiri reports one alert, at line 1, column 5

#### Scenario: An exclusion in one item

- **WHEN** a rule has `scope: [heading, 'paragraph & ~link']` and `tokens: [marker]`, and the input is a heading `# A [marker](https://x.example)` and a paragraph `See [marker](https://x.example).`
- **THEN** hakkiri reports one alert, on the heading

### Requirement: A check reads text without inline markup

The text a check reads SHALL hold the text of strong, emphasis and link elements without their markers, and no link target.

#### Scenario: A phrase across markup

- **WHEN** a rule has `tokens: ['bold link']`, and the input is `A **bold** [link](https://a.example) here.`
- **THEN** hakkiri reports one alert, with the matched text `bold link`, at line 1, column 5

### Requirement: A match on a code span is not reported

A match that overlaps a code span SHALL give no alert from `existence` or `substitution`.

#### Scenario: A pattern that reaches into a code span

- **WHEN** a rule has `tokens: ['run \w+']`, and the input is ``Then run `cargo test` now.``
- **THEN** hakkiri reports no alert

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

#### Scenario: A quote inside an item

- **WHEN** a rule has `scope: paragraph`, and the input is a list item that holds a quote of the paragraphs `Para one.` and `Para two.`
- **THEN** hakkiri selects one block, `Para two.`, and a rule with `scope: list` selects `Para one.`

#### Scenario: A paragraph after a thematic break in a quote

- **WHEN** a rule has `scope: blockquote`, and the input is a quote that holds a thematic break and then the paragraph `After the rule.`
- **THEN** hakkiri selects one block, `After the rule.`

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

[rule-files]: ../rule-files/spec.md
