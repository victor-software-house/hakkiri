# Spec Delta

## Purpose

A `metric` rule computes a formula over counts of the text it measures, and reports when a condition on the result holds.  
Its variables, formula and condition keep the meaning Vale 3.26.0 gives them, plus `occurrences`.

## ADDED Requirements

### Requirement: A metric reports when the condition holds

A `metric` rule SHALL evaluate its `formula`, then its `condition` against the result, and report one alert for each measured text where the condition holds.  
A `%s` in its message SHALL read as the result with two decimals.

#### Scenario: Words per sentence

- **WHEN** a rule has `extends: metric`, `message: "%s words per sentence"`, `formula: words / sentences` and `condition: '>= 4'`, and the input is `One two three four five six. Seven eight.`
- **THEN** hakkiri reports one alert, with the message `4.00 words per sentence`

### Requirement: The formula reads Vale's text counts

The formula SHALL read `words`, `sentences`, `characters`, `syllables`, `complex_words`, `long_words`, `polysyllabic_words`, `quote_words` and `sentence_length_sd`, counted as Vale 3.26.0 counts them.  
A word SHALL be a run of letters, digits and apostrophes, or initials such as `U.S.`

#### Scenario: A contraction and initials

- **WHEN** a rule has `message: "%s"`, `formula: words` and `condition: '> 0'`, and the input is `Don't stop the U.S. team.`
- **THEN** hakkiri reports one alert, with the message `5.00`

#### Scenario: Characters are letters and digits

- **WHEN** the rule has `formula: characters` and reads `Ab, 12 c.`
- **THEN** hakkiri reports one alert, with the message `5.00`

### Requirement: The formula reads readability scores

The formula SHALL read `automated_readability`, `coleman_liau`, `dale_chall`, `flesch_kincaid`, `flesch_reading_ease`, `gunning_fog`, `lix` and `smog`, computed as Vale 3.26.0 computes them, when the text has at least one sentence.

#### Scenario: A short sentence

- **WHEN** a rule has `message: "%s"`, `formula: flesch_kincaid` and `condition: '< 0'`, and the input is `The cat sat on the mat.`
- **THEN** hakkiri reports one alert, with the message `-1.45`

### Requirement: The formula reads element counts

The formula SHALL read `paragraphs`, `list`, `blockquote`, `pre` and `heading_h1` to `heading_h6` as the number of those elements in the file, and `heading.h1` SHALL read as `heading_h1`.  
In a metric with a scope, each element count SHALL be 0.

#### Scenario: Second-level headings

- **WHEN** a rule has `message: "%s"`, `formula: heading.h2` and `condition: '> 1'`, and the input holds three `##` headings, each followed by a paragraph
- **THEN** hakkiri reports one alert, with the message `3.00`

### Requirement: With no scope, a metric measures the prose of the file

A `metric` rule with no `scope`, or with `scope: text`, SHALL measure the paragraphs, list items and block quotes of the file together.  
It SHALL leave out headings and table cells.  
Text with no words SHALL give no alert.  
A `metric` rule with another `scope` SHALL measure each block that scope selects, and report at the start of that block.

#### Scenario: A heading is not measured

- **WHEN** a rule has `message: "%s"`, `formula: words` and `condition: '>= 0'`, and the input is `# One two three` followed by a paragraph `Four five.`
- **THEN** hakkiri reports one alert, with the message `2.00`

#### Scenario: Only a heading

- **WHEN** the same rule reads a file that holds only `# One two three`
- **THEN** hakkiri reports no alert

#### Scenario: A scope measures each block

- **WHEN** a rule has `message: "%s"`, `scope: paragraph`, `formula: words` and `condition: '> 1'`, and the input is the paragraphs `One two three.`, `Four.` and `Five six.`
- **THEN** hakkiri reports two alerts, with the messages `3.00` and `2.00`

### Requirement: Formulas use Tengo's expression syntax

A formula and a condition SHALL accept numbers, variables, parentheses, `+`, `-`, `*`, `/`, `%`, comparisons, `&&`, `||`, `!`, the functions of Tengo's `math` module written `math.sqrt(x)`, and `round(x)` or `round(x, places)`.  
A condition MAY read `result`.

#### Scenario: A two-sided condition

- **WHEN** a rule has `message: "%s"`, `formula: round(words / 3, 1)` and `condition: '> 0.5 && result < 1'`, and the input is `One two.`
- **THEN** hakkiri reports one alert, with the message `0.70`

### Requirement: A formula that cannot run is refused

hakkiri SHALL refuse a `metric` rule whose `formula` or `condition` does not parse or names an unknown variable.  
The error SHALL name the rule and the field.

#### Scenario: A formula cut short

- **WHEN** rule `Team.Rate` has `extends: metric` and `formula: words /`
- **THEN** hakkiri reports an error naming `Team.Rate` and `formula`, and exits with a failure status
