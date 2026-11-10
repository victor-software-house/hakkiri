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
