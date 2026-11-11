# vocabularies Specification

## Purpose

A vocabulary is a folder of word lists in a package, which a project names in `vocab` at its top level and in a glob.  
It holds the terms a project accepts, and the checks hold back the matches those terms name, as Vale's `Vocab` does.

## Requirements

### Requirement: A vocabulary is a folder of word lists in a package

hakkiri SHALL read a named vocabulary from `config/vocabularies/<name>/` under a loaded package folder, and SHALL read every `accept.txt` and `reject.txt` below that folder and no other file.

#### Scenario: A list in a subfolder

- **WHEN** the vocabulary `Team` holds `accept.txt` with `GitHub`, `sub/accept.txt` with `Kubernetes`, and `words.txt` with `Gamma`
- **THEN** `GitHub` and `Kubernetes` are accepted terms and `Gamma` is no term ([vocabularies 3][vocabularies-3])

#### Scenario: A vocabulary that is not there

- **WHEN** the config names `Nope` and no loaded package holds `config/vocabularies/Nope`
- **THEN** hakkiri reports an error that names `Nope` and the line of the key, and exits with a failure status

#### Scenario: Two packages hold one name

- **WHEN** two packages each hold `config/vocabularies/Team`
- **THEN** hakkiri reports an error that names both folders, and exits with a failure status

### Requirement: A line of a list is a term

hakkiri SHALL read each line of a list as one term after trimming it, and SHALL skip a blank line and a line that starts with a hash and a space.

#### Scenario: A comment and a term that looks like one

- **WHEN** a list holds the lines `# a comment`, `#nocomment`, `padded` with spaces around it, and a blank line
- **THEN** the terms are `#nocomment` and `padded`

#### Scenario: A line ending of Windows

- **WHEN** a list holds `Alpha` followed by a carriage return and a line feed
- **THEN** the term is `Alpha`

### Requirement: A term is plain text or a pattern

hakkiri SHALL read a term that holds no pattern operator other than a period as plain text, with each period literal, and any other term as a pattern.
A run of whitespace in a term SHALL match any run of whitespace.

#### Scenario: A plain term with a period

- **WHEN** `Node.js` is accepted, `Team.Odd` is an `existence` rule with the token `Nodexjs`, and the text is `Nodexjs and Node.js.`
- **THEN** the only alert is ``warning Team.Odd: Remove 'Nodexjs'.``, because the period of `Node.js` is literal

#### Scenario: A pattern

- **WHEN** `harness(?:es)?` is accepted, `Team.Filler` is an `existence` rule with the token `harnesses`, and the text is `Use harnesses.`
- **THEN** `Team.Filler` reports nothing, and it reports ``warning Team.Filler: Remove 'harnesses'.`` with no vocabulary

#### Scenario: A phrase that wraps

- **WHEN** `mea culpa` is accepted and a text holds `mea` and `culpa` on two lines
- **THEN** the pair counts as the phrase

#### Scenario: A term that is not a pattern

- **WHEN** a list holds `C(` on line 3
- **THEN** hakkiri reports an error that names the file and line 3, and exits with a failure status, where Vale reports the pattern error with no line ([vocabularies 4][vocabularies-4])

### Requirement: Accepted terms hold back matches of the checks that read them

hakkiri SHALL add the accepted terms to the exceptions of each `capitalization`, `conditional`, `existence`, `repetition` and `substitution` rule, before it rewrites and sorts them.
It SHALL hold back a match that is an accepted term or lies inside an accepted phrase.
A rule with `vocab: false` SHALL read none, and an `existence` rule with `nonword` SHALL read none.

#### Scenario: An existence rule

- **WHEN** `Team.Filler` is an `existence` rule with the tokens `really`, `GitHub`, `harness`, `mea` and `culpa`, the vocabulary accepts `GitHub`, `harness(?:es)?` and `mea culpa`, and the text is `Use GitHub and harness. The mea culpa stands, and mea alone.`
- **THEN** the only alert is ``warning Team.Filler: Remove 'mea'.`` for the lone `mea`, and with no vocabulary every token draws one

#### Scenario: A capitalization rule

- **WHEN** a `capitalization` rule with `match: $sentence` checks the heading `# Using Kubernetes with Terraform today`, and the vocabulary accepts `Kubernetes` and `Terraform`
- **THEN** it reports nothing, and it reports `'Using Kubernetes with Terraform today' should be in sentence case` when the rule sets `vocab: false` or no vocabulary is named

#### Scenario: A heading with words nobody accepted

- **WHEN** the same rule checks `# Using Rust with Docker today`
- **THEN** it reports `'Using Rust with Docker today' should be in sentence case`, vocabulary or not

### Requirement: Spelling reads the accepted terms whole and without case

hakkiri SHALL add each accepted term to the exceptions of a `spelling` rule as a whole word matched without regard to case, with no `vocab` field to turn that off.
It SHALL mask an accepted phrase before it checks the words.

#### Scenario: An accepted word and a near miss

- **WHEN** `kubectl` is accepted and a `spelling` rule checks `Run kubectl and kubectx now.`
- **THEN** it reports only `kubectx`

#### Scenario: A phrase and its lone word

- **WHEN** `mea culpa` is accepted and the text is `The mea culpa stands, and mea alone.`
- **THEN** a `spelling` rule that flags `mea` reports the lone `mea` once

### Requirement: A glob adds vocabularies for its files

hakkiri SHALL hold back, in a file, a match of an `existence`, `repetition`, `spelling` or `substitution` rule.
The match SHALL be an accepted term of a vocabulary that a matching glob names, or lie inside an accepted phrase of one.
It SHALL do so whatever the rule's `vocab` and `nonword` say, and a `capitalization` or `conditional` rule SHALL read none.
A file SHALL take the vocabularies of every glob that matches it, besides the top-level ones.

#### Scenario: A glob's vocabulary

- **WHEN** a glob names `Team`, `Team.Filler0` is an `existence` rule with `vocab: false` and the token `GitHub`, and the text is `Use GitHub here.`
- **THEN** it reports nothing, and with `Team` named at the top level it reports ``warning Team.Filler0: Remove 'GitHub'.``

#### Scenario: Rules a glob does not reach

- **WHEN** a glob names `Team` and accepts `API` and `Kubernetes`, a `conditional` rule reads `The SDK and the API are here.` and a `capitalization` rule with `match: $sentence` reads `# Using Kubernetes with Terraform today`
- **THEN** the `conditional` rule reports `SDK` and `API`, and the `capitalization` rule reports the heading, where the top level holds both back

#### Scenario: Two globs and the top level

- **WHEN** the top level names `Docs`, which accepts `Zebra`, one glob names `Team`, which accepts `GitHub`, and the text is `We saw Zebra and GitHub here.`
- **THEN** `Team.Filler` with the tokens `Zebra` and `GitHub` reports nothing, and a second glob that names `Docs` for another file does not reach this one

#### Scenario: Spelling reads a glob's terms exactly

- **WHEN** `zorbl` is accepted, and a `spelling` rule reads `Zorbl, zorbl and ZORBL.`
- **THEN** a vocabulary at the top level holds back all three words, and a glob's vocabulary holds back `zorbl` only, and the rule reports ``Did you really mean 'Zorbl'?``

[vocabularies-3]: ../../changes/archive/2026-11-11-vocabularies/design.md#3-a-vocabulary-is-a-folder-in-a-loaded-package
[vocabularies-4]: ../../changes/archive/2026-11-11-vocabularies/design.md#4-a-line-of-a-list-is-a-pattern
