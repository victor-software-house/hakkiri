# sequence-check Specification

## Purpose

A `sequence` rule matches a run of words in one sentence, each word tested by a token's pattern, part-of-speech tag or universal tag.  
Its tokens keep the meaning Vale 3.26.0 gives them.

## Requirements

### Requirement: sequence matches tokens word by word

A `sequence` rule SHALL split each selected sentence into words and part-of-speech tags.  
It SHALL report each run of words that its `tokens` match in order, as one alert that spans the run.  
A token's `pattern` SHALL match a whole word, and its `tag` SHALL match anywhere in the word's Penn Treebank tag.

In its message, each `%s` SHALL read as the next matched word.

#### Scenario: Two words as patterns

- **WHEN** a rule has `extends: sequence`, `message: "Write '%s%s' as one word."` and `tokens: [{pattern: your}, {pattern: self}]`, and the input is `Do it your self.`
- **THEN** hakkiri reports one alert, with the matched text `your self` and the message `Write 'yourself' as one word.`

#### Scenario: A pattern is a whole word

- **WHEN** the same rule reads `Value your self-worth.`
- **THEN** hakkiri reports no alert

#### Scenario: A tag and a pattern

- **WHEN** a rule has `message: "Passive: '%s %s'."` and `tokens: [{tag: MD}, {pattern: be}]`, and the input is `The file can be read.`
- **THEN** hakkiri reports one alert, with the matched text `can be` and the message `Passive: 'can be'.`

### Requirement: sequence reads sentences only

A `sequence` rule SHALL read the sentences of the nodes its `scope` selects, or of every prose node when it has no scope.  
A match SHALL never cross a sentence end.

#### Scenario: Sentences of list items

- **WHEN** the first rule above has `scope: list`, and the input holds the paragraph `Do it your self.` on line 1 and the list item `- Do it your self.` on line 3
- **THEN** hakkiri reports one alert, on line 3

### Requirement: skip and min widen a token

A token with `skip: N` SHALL match up to N words of any kind before the word it names.  
A token with `min: N` SHALL repeat N times, each with its own `skip`.  
With `ignorecase: true`, patterns SHALL ignore case.

#### Scenario: Words between two tokens

- **WHEN** a rule has `message: "Drop 'then'."`, `ignorecase: true` and `tokens: [{pattern: if}, {pattern: then, skip: 2}]`, and the input is `If it rains then stay.`
- **THEN** hakkiri reports one alert, with the matched text `If it rains then`

#### Scenario: Too many words between

- **WHEN** the same rule reads `If it rains all day then stay.`
- **THEN** hakkiri reports no alert

#### Scenario: A word twice

- **WHEN** a rule has `message: "Repeated."` and `tokens: [{pattern: very, min: 2}]`, and the input is `This is very very good.`
- **THEN** hakkiri reports one alert, with the matched text `very very`

### Requirement: negate and target

A token with `negate: true` SHALL match a word its pattern or tag does not match.  
A negated token SHALL also hold at the start or end of the sentence.  
When one or more tokens set `target: true`, the alert SHALL span only the words from the first to the last of them.

#### Scenario: Not followed by a noun

- **WHEN** a rule has `message: "No noun."` and `tokens: [{pattern: the}, {tag: NN, negate: true}]`, and the input is `Pick the red one.`
- **THEN** hakkiri reports one alert, with the matched text `the red`

#### Scenario: A negated token at the end

- **WHEN** the same rule has `scope: heading` and reads `# Open the`
- **THEN** hakkiri reports one alert, with the matched text `the`

#### Scenario: Point at one word

- **WHEN** a rule has `message: "Join it."` and `tokens: [{pattern: your}, {pattern: self, target: true}]`, and the input is `Do it your self.`
- **THEN** hakkiri reports one alert, with the matched text `self`

### Requirement: upos names a universal tag

A token's `upos` SHALL name one or more universal part-of-speech tags joined by `|`, matched through the Penn Treebank tags Vale maps them to.  
hakkiri SHALL refuse a token that sets both `tag` and `upos`, or names an unknown universal tag.

#### Scenario: A universal noun

- **WHEN** a rule has `message: "Found."` and `tokens: [{pattern: the}, {upos: NOUN}]`, and the input is `Open the file.`
- **THEN** hakkiri reports one alert, with the matched text `the file`

#### Scenario: Both tag kinds

- **WHEN** rule `Team.Noun` has a token `{tag: NN, upos: NOUN}`
- **THEN** hakkiri reports an error naming `Team.Noun`, and exits with a failure status

### Requirement: exceptions drop a match

A match that begins inside a match of one of the rule's `exceptions` patterns SHALL give no alert.

#### Scenario: A quoted phrase

- **WHEN** the first rule above has `exceptions: ['"[^"]*"']`, and the input is `He wrote "your self" twice.`
- **THEN** hakkiri reports no alert

### Requirement: model is refused

hakkiri SHALL refuse a `sequence` rule that sets `model`, since it has no format for another tagger's model.  
The error SHALL name the rule and the field.

#### Scenario: A rule with a model

- **WHEN** rule `Team.Tags` has `extends: sequence` and `model: other`
- **THEN** hakkiri reports an error naming `Team.Tags` and `model`, and exits with a failure status
