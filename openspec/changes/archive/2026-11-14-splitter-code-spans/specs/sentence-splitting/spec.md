## ADDED Requirements

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
