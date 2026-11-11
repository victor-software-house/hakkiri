# Spec Delta

## Purpose

A `capitalization` rule compares the case of each selected text with the case its `match` asks for, and reports text that falls short.  
Its case styles, word share and exceptions keep the meaning Vale 3.26.0 gives them.

## ADDED Requirements

### Requirement: capitalization reports text in the wrong case

A `capitalization` rule SHALL convert each selected text to the case its `match` names.  
When the text is not in that case, it SHALL report one alert that spans the text.  
In its message, the first `%s` SHALL read as the text and the second as the converted text.

#### Scenario: A heading in title case

- **WHEN** a rule has `extends: capitalization`, `message: "'%s' should be '%s'."`, `match: $sentence` and `scope: heading`, and the input is `# Getting Started With Hakkiri`
- **THEN** hakkiri reports one alert at line 1, with the matched text `Getting Started With Hakkiri` and the message `'Getting Started With Hakkiri' should be 'Getting started with hakkiri'.`

### Requirement: sentence case counts the words that agree

With `match: $sentence`, a word in capitals of two letters or more SHALL keep its case.  
Of the other words, the first SHALL start with a capital and the rest SHALL be in lower case.

The text SHALL pass when its first word passes and enough words match the converted text.  
The share that agrees SHALL be at least `threshold`, 0.8 by default.

#### Scenario: An acronym

- **WHEN** the rule above reads `# Install the CLI`
- **THEN** hakkiri reports no alert

#### Scenario: A first word in lower case

- **WHEN** the rule above reads `# getting started`
- **THEN** hakkiri reports one alert

#### Scenario: Three words of four agree

- **WHEN** the rule above reads `# Note: Run the tests`
- **THEN** hakkiri reports one alert, since 0.75 is below 0.8

#### Scenario: A lower threshold

- **WHEN** the rule above has `threshold: 0.7` and reads `# Note: Run the tests`
- **THEN** hakkiri reports no alert

### Requirement: An indicator capitalizes the next word

With `match: $sentence`, a word that ends with one of the rule's `indicators` SHALL make the next word start with a capital.  
So SHALL a first word that is a number followed by a period.

#### Scenario: A colon as indicator

- **WHEN** the rule above has `indicators: [':']` and reads `# Note: Run the tests`
- **THEN** hakkiri reports no alert

### Requirement: exceptions keep their own case

A word that matches one of the rule's `exceptions` SHALL keep the case the exception gives it, and SHALL count as agreeing.

#### Scenario: A product name

- **WHEN** the rule above has `exceptions: [Hakkiri]` and reads `# Getting started with Hakkiri`
- **THEN** hakkiri reports no alert

### Requirement: title case follows AP or Chicago

With `match: $title`, the first and last words SHALL start with a capital.  
In `AP` style, the default, the articles, conjunctions and prepositions of Vale's short word list SHALL be in lower case.  
In `AP` style, `to` SHALL start with a capital unless the next word is tagged as a noun.

In `Chicago` style, the short words and Vale's list of longer prepositions SHALL be in lower case.

#### Scenario: A preposition of four letters

- **WHEN** a rule has `message: "'%s' should be '%s'."`, `match: $title`, `style: Chicago` and `scope: heading`, and reads `# Notes From the Field`
- **THEN** hakkiri reports one alert, with the message `'Notes From the Field' should be 'Notes from the Field'.`

#### Scenario: The same heading in AP style

- **WHEN** the rule has `style: AP` and reads `# Notes From the Field`
- **THEN** hakkiri reports no alert

#### Scenario: to before an article

- **WHEN** the rule has `style: AP` and reads `# Welcome to the Team`
- **THEN** hakkiri reports one alert, with the message `'Welcome to the Team' should be 'Welcome To the Team'.`

#### Scenario: to before a noun

- **WHEN** the rule has `style: AP` and reads `# A Trip to Paris`
- **THEN** hakkiri reports no alert

### Requirement: lower, upper and a pattern

With `match: $lower` or `match: $upper`, the text SHALL pass only when it equals its lower or upper case form, or matches an exception.  
Any other `match` SHALL be a pattern that the text passes by matching.

#### Scenario: A heading that is not lower case

- **WHEN** a rule has `message: "'%s' should be '%s'."`, `match: $lower` and `scope: heading`, and reads `# Release Notes`
- **THEN** hakkiri reports one alert, with the message `'Release Notes' should be 'release notes'.`

#### Scenario: A pattern

- **WHEN** the rule above has `match: '^[A-Z][a-z]+$'` instead, and reads the heading `# Changelog` on line 1 and `# ChangeLog` on line 3
- **THEN** hakkiri reports one alert, on line 3

### Requirement: An unknown style is refused

A `capitalization` rule whose `style` is neither `AP` nor `Chicago` SHALL be refused with an error that names the rule and the value.

#### Scenario: A misspelt style

- **WHEN** rule `Team.Headings` has `extends: capitalization`, `match: $title` and `style: chicago`
- **THEN** hakkiri refuses the rule with an error naming `Team.Headings` and `chicago`
