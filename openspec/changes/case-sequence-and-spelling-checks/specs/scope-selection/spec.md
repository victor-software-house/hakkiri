# Spec Delta

## MODIFIED Requirements

### Requirement: sentence selects each sentence

The scope `sentence` SHALL select each sentence of a paragraph, a heading or a table cell.  
A sentence SHALL keep the nodes around its node, so an exclusion such as `~blockquote` applies to it, and a part such as `list` selects the sentences inside a list item.

#### Scenario: Two sentences measured apart

- **WHEN** a rule has `extends: metric`, `message: "%s words"`, `scope: sentence`, `formula: words` and `condition: '> 3'`, and the input is `One two. Three four five six.`
- **THEN** hakkiri reports one alert, with the message `4.00 words`

#### Scenario: A sentence in a list item

- **WHEN** a rule has `scope: sentence & list` and the input holds the paragraph `One two.` and the list item `- Three four.`
- **THEN** hakkiri selects one sentence, `Three four.`

### Requirement: paragraph leaves out list items and quotes

The scope `paragraph` SHALL select a paragraph that has no list item or block quote around it.  
Vale 3.26.0 also selects the paragraphs after the first inside an item or a quote, and hakkiri does not.

#### Scenario: A paragraph and a list item

- **WHEN** a rule has `scope: paragraph` and `tokens: [marker]`, and the input holds `- A marker item.` and a paragraph `A marker.`
- **THEN** hakkiri reports one alert, in the paragraph

#### Scenario: A later paragraph of a list item

- **WHEN** the same rule reads the list item `- First.` followed by the paragraph `A marker here.`, indented under it
- **THEN** hakkiri reports no alert

## ADDED Requirements

### Requirement: A part reads an element and everything inside it

A scope part that names a list item or a block quote SHALL select the text of everything inside it: a nested list, quote, heading or table, and the later paragraphs of an item.  
Vale 3.26.0 gives each text block only the scope of its own element.

#### Scenario: A heading in a quote

- **WHEN** a rule has `scope: blockquote` and `tokens: [marker]`, and the input is `> # A marker heading`
- **THEN** hakkiri reports one alert

#### Scenario: A later paragraph of a list item

- **WHEN** a rule has `scope: list` and `tokens: [marker]`, and the input is the list item `- First.` followed by the paragraph `A marker here.`, indented under it
- **THEN** hakkiri reports one alert
