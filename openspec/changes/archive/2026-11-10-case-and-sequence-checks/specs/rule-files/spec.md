# Spec Delta

## RENAMED Requirements

- FROM: `### Requirement: An exclusion reaches any depth`
- TO: `### Requirement: An exclusion leaves out blocks of that scope`

## MODIFIED Requirements

### Requirement: An exclusion leaves out blocks of that scope

A scope part written `~` and a name SHALL leave out each block of text that has that scope, and no other.  
A block has the scope of one element, as the scope-selection spec fixes, so an element inside a quote keeps its own scope: a list item after text in a quote is not left out by `~blockquote`.

#### Scenario: A sentence in a list inside a quote

- **WHEN** a rule has `scope: sentence & ~blockquote` and `tokens: [marker]`, and the input is `A marker here.` followed by a quote that holds `> - A marker in a list.`
- **THEN** hakkiri reports one alert, on line 1, and none on the line of the list item

#### Scenario: A list item after text in a quote

- **WHEN** the same rule reads `A marker here.` followed by a quote that holds `> Quote text.`, an empty quoted line, and `> - A marker in a list.`
- **THEN** hakkiri reports two alerts, on line 1 and on the line of the list item
