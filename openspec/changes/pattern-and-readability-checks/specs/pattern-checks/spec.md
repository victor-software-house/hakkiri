# Spec Delta

## Purpose

The pattern checks compare matches with each other: `consistency` with the other form of a pair, `conditional` with a definition, and `repetition` with the match before it.  
Each keeps the meaning Vale 3.26.0 gives it.

## ADDED Requirements

### Requirement: consistency reports a file that uses both forms

A `consistency` rule SHALL match each `either` key and value as an `existence` rule matches tokens.  
A node with a match of the pair SHALL give one alert when both forms have matched in the file so far, that node included.  
That alert SHALL sit at the last match of the pair in the node.

#### Scenario: Two spellings in two paragraphs

- **WHEN** a rule has `extends: consistency`, `message: "Inconsistent: '%s'"` and `either: {advisor: adviser}`, and the input holds a paragraph `An advisor came.` and then `The adviser left.`
- **THEN** hakkiri reports one alert, in the second paragraph, with the message `Inconsistent: 'adviser'`

#### Scenario: One spelling throughout

- **WHEN** the same rule reads `An advisor came.` and `The advisor left.`
- **THEN** hakkiri reports no alert

### Requirement: consistency checks each pair alone

Each pair of `either` SHALL be checked on its own, and an alert SHALL carry the rule's name, not the name of the pair.

#### Scenario: Two pairs, one mixed

- **WHEN** rule `Team.Spelling` has `either: {advisor: adviser, color: colour}`, and the input is `The advisor likes color. The adviser likes color.`
- **THEN** hakkiri reports one alert from `Team.Spelling`, with the matched text `adviser`

### Requirement: conditional without a group needs second in the node

When `second` has no capture group, a `conditional` rule SHALL report each match of `first` in a selected node where `second` does not match.

#### Scenario: A term with no definition nearby

- **WHEN** a rule has `extends: conditional`, `message: "Define '%s'."`, `first: '\bAPI\b'` and `second: 'application programming interface'`, and the input is `Call the API.`
- **THEN** hakkiri reports one alert, with the message `Define 'API'.`

### Requirement: conditional with a group keeps the captured terms

When `second` has a capture group, each text a group captures SHALL count as defined from that node to the end of the file.  
A match of `first` that is not defined, and is not an `exceptions` entry, SHALL give one alert.

#### Scenario: An acronym defined once

- **WHEN** a rule has `first: '\b[A-Z]{3,5}\b'` and `second: '\(([A-Z]{3,5})\)'`, and the input holds `The World Health Organization (WHO) met. WHO agreed.` and then `The CDC said no.`
- **THEN** hakkiri reports one alert, with the matched text `CDC`

#### Scenario: A definition after the use

- **WHEN** the same rule reads `The CDC said no.` and then `The Centers for Disease Control (CDC) spoke.`
- **THEN** hakkiri reports one alert, on line 1

### Requirement: repetition reports a repeated match

A `repetition` rule SHALL compare each match of its `tokens` with the match before it, ignoring case when `ignorecase` is `true`.  
It SHALL report two equal matches as one alert that spans both.  
With `alpha: true`, only matches made of letters SHALL count.

#### Scenario: A doubled word

- **WHEN** a rule has `extends: repetition`, `message: "'%s' is repeated!"`, `ignorecase: true`, `alpha: true` and `tokens: ['[^\s.!?]+']`, and the input is `This is is a test.`
- **THEN** hakkiri reports one alert, with the message `'is' is repeated!` and the matched text `is is`

#### Scenario: A repeat across two sentences

- **WHEN** the same rule reads `I almost forgot about that. That is important.`
- **THEN** hakkiri reports no alert

### Requirement: repetition counts repeats up to max

A `repetition` rule SHALL count the repeats in a selected node and report a repeat only when the count is over `max`.  
The count SHALL go back to 0 after each alert, as Vale 3.26.0 counts it.  
A match that does not repeat SHALL leave the count as it is.

#### Scenario: Two separate repeats with max 1

- **WHEN** the rule above has `max: 1` and reads `The the cat saw a a dog.`
- **THEN** hakkiri reports one alert, with the matched text `a a`

### Requirement: repetition holds back an exception

A `repetition` rule SHALL NOT report a repeat when the text it spans matches an `exceptions` entry.

#### Scenario: A repeat that is a phrase

- **WHEN** the rule above has `exceptions: ['had had']` and reads `She had had enough.`
- **THEN** hakkiri reports no alert

### Requirement: conditional honours ignorecase

When `ignorecase` is `true`, a `conditional` rule SHALL match `first` and `second` without regard to case.

#### Scenario: A definition written in title case

- **WHEN** a rule has `extends: conditional`, `message: "Define '%s'."`, `ignorecase: true`, `first: '\bAPI\b'` and `second: 'application programming interface'`, and the input is `The Application Programming Interface is stable. Call the API.`
- **THEN** hakkiri reports no alert

### Requirement: conditional compares a defined term without regard to case too

When `ignorecase` is `true`, a match of `first` SHALL count as defined when it equals a captured term without regard to case.

#### Scenario: A term defined in capitals

- **WHEN** a rule has `ignorecase: true`, `first: '\b[A-Za-z]{3}\b'` and `second: '\(([A-Z]{3})\)'`, and the input is `Centers (CDC) met. The cdc agreed.`
- **THEN** hakkiri reports two alerts, with the matched texts `met` and `The`

### Requirement: the pattern checks ignore code

A match that overlaps a code span SHALL NOT count for `consistency`, `conditional` or `repetition`.  
It SHALL be no form seen, give no alert, define nothing, and leave the next match nothing before it to repeat.

#### Scenario: A repeat across code

- **WHEN** the doubled-word rule reads ``This `is` is a test.``
- **THEN** hakkiri reports no alert

#### Scenario: A form in code

- **WHEN** the spelling rule reads `An advisor came.` and then ``The `adviser` left.``
- **THEN** hakkiri reports no alert

#### Scenario: A definition in code

- **WHEN** the acronym rule reads ``Run `(CDC)` first.`` and then `The CDC said no.`
- **THEN** hakkiri reports one alert, with the matched text `CDC`

### Requirement: the pattern checks need something to match

hakkiri SHALL refuse a `consistency` rule whose `either` holds no pair or a form with no text, a `repetition` rule whose `tokens` hold no text, and a `conditional` rule whose `first` or `second` holds no text.  
The error SHALL name the field.

#### Scenario: No pair

- **WHEN** a rule has `extends: consistency` and no `either`
- **THEN** hakkiri reports an error naming `either`, and exits with a failure status

### Requirement: conditional refuses a View scope

hakkiri has no Views, so it SHALL refuse a `conditional` rule that sets `in`, with an error that names the rule and the field.

#### Scenario: A rule that looks in a View

- **WHEN** rule `Team.Defined` has `extends: conditional` and `in: glossary`
- **THEN** hakkiri reports an error naming `Team.Defined` and `in`, and exits with a failure status
