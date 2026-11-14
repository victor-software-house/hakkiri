# vocabularies Specification

## Purpose

A vocabulary is a folder of word lists in a package, which a project names in `vocab` at its top level and in a glob.  
It holds the terms a project accepts, and the checks hold back the matches those terms name, as Vale's `Vocab` does.

## Requirements

### Requirement: A vocabulary is a folder of word lists in a package

hakkiri SHALL read a named vocabulary from `config/vocabularies/<name>/` under a loaded package folder, and SHALL read every `accept.txt`, `reject.txt`, `vocab.yml` and `vocab.yaml` below that folder and no other file.

#### Scenario: A list in a subfolder

- **WHEN** the vocabulary `Team` holds `accept.txt` with `GitHub`, `sub/accept.txt` with `Kubernetes`, and `words.txt` with `Gamma`
- **THEN** `GitHub` and `Kubernetes` are accepted terms and `Gamma` is no term ([vocabularies 3][vocabularies-3])

#### Scenario: A vocabulary that is not there

- **WHEN** the config names `Nope` and no loaded package holds `config/vocabularies/Nope`
- **THEN** hakkiri reports an error that names `Nope` and the line of the key, and exits with a failure status

#### Scenario: Two packages hold one name

- **WHEN** two packages each hold `config/vocabularies/Team`
- **THEN** hakkiri reports an error that names both folders, and exits with a failure status

#### Scenario: Text and YAML files together

- **WHEN** the vocabulary `Mixed` holds `accept.txt` with `Alpha`, `vocab.yml` that accepts `Beta` and `vocab.yaml` that accepts `Gamma`
- **THEN** `Alpha`, `Beta` and `Gamma` are accepted terms, and `Delta` draws its alert ([yaml-vocabularies 7][yaml-vocabularies-7])

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
It SHALL hold back a match that holds an accepted word as a whole word, with its case, anywhere in the matched text, as Vale does.
It SHALL also hold back a match that lies inside an accepted phrase.
A rule with `vocab: false` SHALL read none, and an `existence` rule with `nonword` SHALL read none.

#### Scenario: An existence rule

- **WHEN** `Team.Filler` is an `existence` rule with the tokens `really`, `GitHub`, `harness`, `mea` and `culpa`, the vocabulary accepts `GitHub`, `harness(?:es)?` and `mea culpa`, and the text is `Use GitHub and harness. The mea culpa stands, and mea alone.`
- **THEN** the only alert is ``warning Team.Filler: Remove 'mea'.`` for the lone `mea`, and with no vocabulary every token draws one

#### Scenario: A match that holds an accepted word

- **WHEN** `Team.Chain` is an `existence` rule with `ignorecase: true` and the raw pattern `with the \w+ in a folder`, the vocabulary accepts `harness(?:es)?`, and the text holds the paragraphs `Run it with the harness in a folder.`, `Run it with the Harness in a folder.`, `Run it with the harnessing in a folder.` and `Run it with the parser in a folder.`
- **THEN** the rule alerts on the second, third and fourth paragraphs and holds back the first, as Vale 3.26.0 does with `ProseStyle.PrepositionChain` and `Vocab = ProseStyle`

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
The match SHALL hold an accepted word of a vocabulary that a matching glob names, as a whole word with its case, or lie inside an accepted phrase of one.
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

### Requirement: A YAML entry is a term, or a term with options

A `vocab.yml` or `vocab.yaml` file SHALL hold the keys `affix`, `accept` and `reject`.
An entry of `accept` or `reject` SHALL be a term, a term mapped to nothing, or a term mapped to the options `regex`, `ignorecase` and `flags`, and SHALL give the patterns that a line of a text list would.

#### Scenario: A plain term

- **WHEN** `accept` holds `GitHub`, `Node.js` and `C++`, a rule flags `GitHub`, `Node.js`, `Nodexjs`, `C++` and `Cxx`, and the text names each
- **THEN** only `Nodexjs` and `Cxx` draw an alert, since a period is literal and `C++` is quoted ([yaml-vocabularies 3][yaml-vocabularies-3])

#### Scenario: A pattern, case and flags

- **WHEN** `accept` holds `harness(?:es)?` with `regex: true`, `zebra` with `ignorecase: true` and `utilize` with `flags: DSG`
- **THEN** `harness` and `harnesses` are accepted, so are `zebra`, `Zebra` and `ZEBRA`, and so are `utilize`, `utilizes`, `utilizing` and `utilized`, but `harnessed` and `utilization` are not

#### Scenario: A term mapped to nothing

- **WHEN** `accept` holds `Kubernetes:` with nothing after the colon
- **THEN** `Kubernetes` is accepted and `kubernetes` is not

#### Scenario: A number as a term

- **WHEN** `accept` holds `3.10` unquoted, and a rule flags `3.10` and `3.1`
- **THEN** only `3.1` draws an alert ([yaml-vocabularies 4][yaml-vocabularies-4])

#### Scenario: Rejected terms

- **WHEN** `reject` holds `ba+r` with `regex: true` and `Baz` with `ignorecase: true`, and the text is `baar and bar, BAZ and baz.`
- **THEN** `Vale.Avoid` reports `baar`, `bar`, `BAZ` and `baz`, each as ``Avoid using '<text>'.``

#### Scenario: A null entry and a YAML 1.1 word

- **WHEN** a list holds `-` alone, and an entry sets `regex: yes`
- **THEN** the empty entry adds nothing and `yes` turns the option on, as in Vale ([yaml-vocabularies 5][yaml-vocabularies-5])

### Requirement: A wrong YAML entry is refused with its file and line

hakkiri SHALL refuse a vocabulary file that has a wrong entry or a missing affix file.
Each problem SHALL name the file and the line Vale names, and one run SHALL report every one.

#### Scenario: A wrong shape

- **WHEN** an entry has more than one key, a key mapped to a string or a list, an unknown option, a `regex` or `ignorecase` that is no boolean, or an empty term
- **THEN** hakkiri refuses the file

#### Scenario: A wrong value

- **WHEN** an entry has both `regex` and `flags`, a pattern that does not compile, flags that do not expand, or an `affix` that names a missing file
- **THEN** hakkiri refuses the file

#### Scenario: Each fault

- **WHEN** a vocabulary holds only one of those faults, in `two-keys`, `scalar-value`, `list-value`, `unknown-option`, `regex-and-flags`, `bad-regex`, `bad-flag`, `empty-term`, `missing-affix`, `bad-bool` or `entry-list` of the fixture
- **THEN** hakkiri exits 2 with ``failed <file>: line N:`` where N is the line of Vale's error

#### Scenario: Several faults in one file

- **WHEN** a file holds an empty term on line 2, an unknown option on line 4 and a pattern that does not compile on line 5
- **THEN** hakkiri reports all three problems, each with its line, where Vale reports the first ([yaml-vocabularies 2][yaml-vocabularies-2])

### Requirement: The affix file comes from the packages

`affix` SHALL name a file `<name>.aff` in the `config/dictionaries` folder of the package that holds the vocabulary, then of every other loaded package in order.
No name, and `en_US-web` with no such file, SHALL read the built-in dictionary.
A name that is not one file name SHALL be refused.

#### Scenario: The holder first

- **WHEN** two packages each hold `config/dictionaries/custom.aff`, with the suffixes `zz` and `qq`, and the package with `zz` holds the vocabulary that names `custom`
- **THEN** `foo` and `foozz` are accepted and `fooqq` is not, whatever the order of the packages

#### Scenario: Another package

- **WHEN** only the other package holds `custom.aff`, with the suffix `qq`
- **THEN** `foo` and `fooqq` are accepted ([yaml-vocabularies 6][yaml-vocabularies-6])

#### Scenario: A name with a folder

- **WHEN** `affix` is `../outside`
- **THEN** hakkiri reports ``affix: `../outside` is not a file name`` with the line of the key

[yaml-vocabularies-2]: ../../changes/archive/2026-11-14-yaml-vocabularies/design.md#2-an-entry-is-checked-on-the-node-and-every-problem-is-reported
[yaml-vocabularies-3]: ../../changes/archive/2026-11-14-yaml-vocabularies/design.md#3-a-plain-term-is-literal
[yaml-vocabularies-4]: ../../changes/archive/2026-11-14-yaml-vocabularies/design.md#4-a-scalar-that-is-not-a-string-keeps-the-spelling-written-in-the-file
[yaml-vocabularies-5]: ../../changes/archive/2026-11-14-yaml-vocabularies/design.md#5-null-and-the-words-of-yaml-11
[yaml-vocabularies-6]: ../../changes/archive/2026-11-14-yaml-vocabularies/design.md#6-the-affix-file-comes-from-the-packages-never-from-dicpath
[yaml-vocabularies-7]: ../../changes/archive/2026-11-14-yaml-vocabularies/design.md#7-every-file-of-the-folder-joins
[vocabularies-3]: ../../changes/archive/2026-11-11-vocabularies/design.md#3-a-vocabulary-is-a-folder-in-a-loaded-package
[vocabularies-4]: ../../changes/archive/2026-11-11-vocabularies/design.md#4-a-line-of-a-list-is-a-pattern
