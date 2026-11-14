# vale-repetition Specification

## Purpose

The built-in style `Vale` holds the rule `Vale.Repetition`, read from Vale's own definition, which flags a word written twice in a row.

## Requirements

### Requirement: The style Vale holds the rule Vale.Repetition

hakkiri SHALL build the rule `Vale.Repetition` into the style `Vale`, with Vale's defaults: `extends: repetition`, the level `error`, the message `'%s' is repeated!`, the scope `text`, `ignorecase`, `alpha`, a `max` of 0 and the tokens `[^\s.!?]+`.
A glob that lists `Vale` SHALL run it, with no vocabulary named.

#### Scenario: A repeated word

- **WHEN** a glob lists `styles: [Vale]` and the text is `This is is a test.`
- **THEN** the only alert is ``error Vale.Repetition: 'is' is repeated!`` at line 1, columns 6 to 10

#### Scenario: Case does not matter

- **WHEN** the text is `The the lower and The The mixed and THE the.`
- **THEN** the alerts are ``'the' is repeated!`` for `The the`, ``'The' is repeated!`` for `The The` and ``'the' is repeated!`` for `THE the`

#### Scenario: A run of three

- **WHEN** the text is `Three three three repeats.`
- **THEN** the alerts are for `Three three` and for `three three`, since the count restarts after an alert, as in Vale ([vale-repetition 5][vale-repetition-5])

#### Scenario: Words that are not letters

- **WHEN** the text is `Numbers 1 1 and don't don't and well-well and under_score under_score.`
- **THEN** there is no alert, since `alpha` reads letters only

#### Scenario: Two sentences and a code span

- **WHEN** the text is `That. That is two sentences.` and a second paragraph is ``A code `x` x and x `x` x.``
- **THEN** there is no alert, since a repeat across two sentences is none and a code span breaks the chain

#### Scenario: The same alerts as Vale

- **WHEN** four Markdown files and a Rust file with doc comments, line comments and block comments are linted
- **THEN** the alerts of the five configs of the fixture are those Vale 3.26.0 gives ([vale-repetition 5][vale-repetition-5])

### Requirement: Words compare by simple case folding

hakkiri SHALL read two words as the same when each character of one folds to the character of the other by simple case folding, as Go's `strings.EqualFold` reads them, and SHALL NOT fold the text as a whole.

#### Scenario: A sigma

- **WHEN** the text is `ΣΑΣ σασ and ΣΑΣ σας and όλος όλος.`
- **THEN** each pair is a repeat, so `ΣΑΣ σασ` is reported, which lowercasing the whole word did not ([vale-repetition 4][vale-repetition-4])

#### Scenario: Other foldings

- **WHEN** the text is `ß ẞ and K k and ǅ ǆ and İstanbul istanbul.`
- **THEN** the first three pairs are repeats, where the Kelvin sign is the first `K`, and `İstanbul istanbul` is none

### Requirement: Accepted terms and the config set the rule

An accepted term of a top-level vocabulary or of a glob's vocabulary SHALL hold a repeat back.
A level, `on` or `off` set by the name `Vale.Repetition` SHALL apply to it as to any rule.

#### Scenario: A vocabulary

- **WHEN** a vocabulary accepts `Bye`, `had had` and `that`, and the text is `She had had enough. Bye bye now. That that is fine.`
- **THEN** none is reported as repeated, at the top level and in a glob, and ``Use 'Bye' instead of 'bye'.`` comes from `Vale.Terms`

#### Scenario: A level and off

- **WHEN** a glob lists `Vale` and sets `Vale.Repetition: warning`, or `Vale.Repetition: off`
- **THEN** the alerts are warnings, or there is none

[vale-repetition-4]: ../../changes/archive/2026-11-14-vale-repetition/design.md#4-words-compare-by-simple-case-folding
[vale-repetition-5]: ../../changes/archive/2026-11-14-vale-repetition/design.md#5-what-the-probe-shows-and-hakkiri-keeps
