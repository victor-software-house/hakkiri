## MODIFIED Requirements

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
