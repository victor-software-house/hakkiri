## ADDED Requirements

### Requirement: A sequence rule may list several forms

A `sequence` rule SHALL take `forms`, a list of mappings that each hold `tokens`, in place of `tokens`.  
It SHALL report each run of words that any form matches, with the rule's own name, and the rest of the rule SHALL apply to every form.

#### Scenario: Either of two forms

- **WHEN** a rule has `message: "'%s'."` and `forms: [{tokens: [{pattern: red}, {pattern: green}]}, {tokens: [{pattern: blue}, {pattern: yellow}]}]`, and the input is `Paint red green or blue yellow.`
- **THEN** hakkiri reports two alerts, with the matched text `red green` and `blue yellow`, both named by the rule

#### Scenario: One form is the same as tokens

- **WHEN** a rule has `forms` of one form with the tokens `[{pattern: red}, {pattern: red}]`, and another rule has `tokens: [{pattern: red}, {pattern: red}]`, and the input is `Paint red red red red.`
- **THEN** both rules report the same three alerts

### Requirement: Overlapping forms give one alert

A match of one form that overlaps a match of another form SHALL give no alert when the other match starts first, or starts at the same word in an earlier form.  
Matches of one form SHALL NOT drop each other.

#### Scenario: Two forms share a word

- **WHEN** a rule has the forms `[{pattern: red}, {pattern: green}]` and `[{pattern: green}, {pattern: blue}]`, and the input is `Paint red green blue.`
- **THEN** hakkiri reports one alert, with the matched text `red green`

### Requirement: A sequence rule holds tokens or forms

hakkiri SHALL refuse a `sequence` rule that sets both `tokens` and `forms`, or neither.  
It SHALL refuse a form with no token that has a `pattern`, `tag` or `upos` without `negate`, and name the form's tokens in the error.

#### Scenario: Both keys

- **WHEN** a rule has `tokens: [{pattern: red}]` and `forms: [{tokens: [{pattern: green}]}]`
- **THEN** hakkiri reports an error that says `tokens` and `forms` are both set, and exits with a failure status

#### Scenario: A fault in the second form

- **WHEN** the second of two forms has the tokens `[{pattern: green}, {pattern: '('}]`
- **THEN** hakkiri reports an error naming `forms[1].tokens[1].pattern`

### Requirement: Forms read a noun, a pronoun and a guard

A form SHALL read the tokens a `tokens` list reads, so a negated token that follows the verb guards the match.

#### Scenario: Three forms of one check

- **WHEN** a rule has a form of a noun and a present verb, a form of a noun, a past verb and a negated `TO|RB|RBR|RBS` token, and a form of a pronoun and a verb
- **AND** the input is `The doc says to run the tool.`, `The jobs asked to end early.` and `The state is open and it knows its names.`
- **THEN** hakkiri reports `doc says` and `it knows`, and no alert for the second sentence
