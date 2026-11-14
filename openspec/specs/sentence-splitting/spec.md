# sentence-splitting Specification

## Purpose

The splitter decides where a sentence ends in the check text of one paragraph, heading or table cell.  
The [scope-selection spec][scope] decides which nodes it reads.

## Requirements

### Requirement: A sentence ends at an end mark before a new sentence

The splitter SHALL end a sentence after `.`, `?` or `!` when whitespace follows and its model reads the next word as a sentence start.

#### Scenario: Two plain sentences

- **WHEN** the splitter reads `The build passed. Ship it now!`
- **THEN** it returns the sentences `The build passed.` and `Ship it now!`

#### Scenario: A question inside a paragraph

- **WHEN** the splitter reads `Does it work? Run the test.`
- **THEN** it returns the sentences `Does it work?` and `Run the test.`

### Requirement: An abbreviation does not end a sentence

The splitter SHALL NOT end a sentence after a word its model holds as an abbreviation, when the next word does not start a sentence.

#### Scenario: A title before a name

- **WHEN** the splitter reads `Ask Dr. Smith first. She knows.`
- **THEN** it returns the sentences `Ask Dr. Smith first.` and `She knows.`

#### Scenario: A Latin abbreviation in the middle

- **WHEN** the splitter reads `Use a list, e.g. a table of flags. Then stop.`
- **THEN** it returns the sentences `Use a list, e.g. a table of flags.` and `Then stop.`

### Requirement: A token with no space holds no boundary

The splitter SHALL NOT end a sentence inside a run of characters with no whitespace, such as a version, a file name or a decimal.

#### Scenario: A version, a file and a price

- **WHEN** the splitter reads `Install v1.2.3 from README.md for 3.50 dollars. Done.`
- **THEN** it returns the sentences `Install v1.2.3 from README.md for 3.50 dollars.` and `Done.`

### Requirement: Closing marks stay with their sentence

A closing quote or bracket right after an end mark SHALL belong to the sentence the end mark ends.  
The splitter SHALL NOT end a sentence inside brackets that close later in the same text.

#### Scenario: A quoted sentence

- **WHEN** the splitter reads `He wrote "Stop." Then he left.`
- **THEN** it returns the sentences `He wrote "Stop."` and `Then he left.`

#### Scenario: A full sentence in brackets

- **WHEN** the splitter reads `Run it twice (the cache needs one pass. The second is fast) and stop.`
- **THEN** it returns one sentence

### Requirement: A list number ends nothing

The splitter SHALL NOT end a sentence after a number or a letter with a period when it starts a sentence or follows a colon.

#### Scenario: Numbered steps

- **WHEN** the splitter reads `Steps: 1. Install it. 2. Run it.`
- **THEN** it returns the sentences `Steps: 1. Install it.` and `2. Run it.`

### Requirement: An ellipsis before a lower-case word does not end a sentence

The splitter SHALL NOT end a sentence at `...` or `…` when the next word starts in lower case.

#### Scenario: A pause inside a sentence

- **WHEN** the splitter reads `Wait... it works now.`
- **THEN** it returns one sentence

### Requirement: The splitter does at least as well as Vale

On each committed evaluation set, the splitter's boundary F1 SHALL be at least the F1 that Vale 3.26.0 scores on it.  
Vale's F1 for each set is committed beside the set.

#### Scenario: The evaluation test

- **WHEN** the test splits every paragraph of the owner's labelled documentation set, the Golden Rules set and the MASC sample
- **THEN** hakkiri's F1 on each set is at least Vale's committed F1 for that set

### Requirement: Training rebuilds the model

The embedded model SHALL be the output of the committed training task on the committed source list.  
The same sources SHALL give the same file.

#### Scenario: A second training run

- **WHEN** the training task runs twice on the same sources
- **THEN** both runs write byte-identical model files

[scope]: ../scope-selection/spec.md

### Requirement: A code span begins and ends a sentence

The splitter SHALL take the byte ranges of the code spans of its text.  
A code span SHALL begin a sentence after a full stop, and a full stop after a code span SHALL end one.  
A code span SHALL NOT be an abbreviation or an initial.

#### Scenario: Two sentences around code spans

- **WHEN** the text is ``Use `a`. Then use `b`. `c` is last.``
- **THEN** the splitter gives three sentences, the last beginning at `` `c` ``

### Requirement: A unit after a number ends a sentence

A unit of measure after a number, followed by a full stop and a capital word, SHALL end a sentence.  
A word that is no unit after a number SHALL end nothing.

#### Scenario: A time in milliseconds

- **WHEN** the text is `The test takes 92 ms. Tests cover a phrase.`
- **THEN** the splitter gives two sentences, the first ending after `ms.`

### Requirement: vs. and a date end nothing

`vs.` SHALL end no sentence, before a capital word or a number.  
A date before a full stop SHALL NOT be a list number.

#### Scenario: Two versions compared

- **WHEN** the text is `Rust 1.99 vs. Go 1.25 differ.`
- **THEN** the splitter gives one sentence
