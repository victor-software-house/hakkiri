# Spec Delta

## Purpose

A `readability` rule averages one or more grade-level scores of the text it measures, and reports when the average is over its `grade`.  
Its scores and counts are the ones the [metric check][metric] reads, so the two checks agree on the same text.

## ADDED Requirements

### Requirement: readability reports a grade over the limit

A `readability` rule SHALL compute each score its `metrics` list names, average them, and report one alert for each measured text where the average is greater than `grade`.  
A `%s` in its message SHALL read as the average with two decimals.

#### Scenario: One score

- **WHEN** a rule has `extends: readability`, `message: "Grade %s."`, `metrics: [Flesch-Kincaid]` and `grade: -2`, and the input is `The cat sat on the mat.`
- **THEN** hakkiri reports one alert, with the message `Grade -1.45.`

#### Scenario: Two scores averaged

- **WHEN** the rule has `metrics: [Flesch-Kincaid, SMOG]` and `grade: 0.5`, and reads `The cat sat on the mat.`
- **THEN** hakkiri reports one alert, with the message `Grade 0.84.`

#### Scenario: A grade under the limit

- **WHEN** the rule has `metrics: [Flesch-Kincaid]` and `grade: 8`, and reads `The cat sat on the mat.`
- **THEN** hakkiri reports no alert

### Requirement: readability knows five scores

The `metrics` list SHALL accept `Automated Readability`, `Coleman-Liau`, `Flesch-Kincaid`, `Gunning Fog` and `SMOG`, each computed as the metric variable of the same score.  
hakkiri SHALL refuse a rule whose list is empty or names any other score, with an error that names the rule and the score.

#### Scenario: A score hakkiri does not know

- **WHEN** rule `Team.Grade` has `extends: readability` and `metrics: [Dale-Chall]`
- **THEN** hakkiri reports an error naming `Team.Grade` and `Dale-Chall`, and exits with a failure status

### Requirement: readability counts a repeated score once

A score listed more than once in `metrics` SHALL count once in the average.

#### Scenario: One score listed twice

- **WHEN** the rule has `metrics: [Flesch-Kincaid, Flesch-Kincaid]` and `grade: -2`, and reads `The cat sat on the mat.`
- **THEN** hakkiri reports one alert, with the message `Grade -1.45.`

### Requirement: readability measures the whole file by default

A `readability` rule with no `scope`, or with `scope: text`, SHALL measure the paragraphs, list items and block quotes of the file together.  
It SHALL measure them as a `metric` rule does, and report at line 1, column 1.  
Text with no sentence SHALL give no alert.

#### Scenario: Two paragraphs measured as one

- **WHEN** a rule has `message: "Grade %s."`, `metrics: [Flesch-Kincaid]` and `grade: -5`, and the input holds the paragraphs `The cat sat.` and `The dog ran.`
- **THEN** hakkiri reports one alert at line 1, column 1, with the message `Grade -2.62.`

#### Scenario: Only a heading

- **WHEN** the same rule reads a file that holds only `# One two three`
- **THEN** hakkiri reports no alert

### Requirement: readability with a scope measures each node

A `readability` rule with any other `scope` SHALL measure each selected node on its own and report at the node's first line, column 1.

#### Scenario: Each paragraph measured

- **WHEN** the rule above has `scope: paragraph` and reads the paragraphs `The cat sat.` and `The dog ran.`
- **THEN** hakkiri reports two alerts, at lines 1 and 3, each with the message `Grade -2.62.`

[metric]: ../../../../../specs/metric-check/spec.md
