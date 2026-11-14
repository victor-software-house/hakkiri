## MODIFIED Requirements

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

## ADDED Requirements

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

[vocabularies-3]: ../../../2026-11-11-vocabularies/design.md#3-a-vocabulary-is-a-folder-in-a-loaded-package
[yaml-vocabularies-2]: ../../design.md#2-an-entry-is-checked-on-the-node-and-every-problem-is-reported
[yaml-vocabularies-3]: ../../design.md#3-a-plain-term-is-literal
[yaml-vocabularies-4]: ../../design.md#4-a-scalar-that-is-not-a-string-keeps-the-spelling-written-in-the-file
[yaml-vocabularies-5]: ../../design.md#5-null-and-the-words-of-yaml-11
[yaml-vocabularies-6]: ../../design.md#6-the-affix-file-comes-from-the-packages-never-from-dicpath
[yaml-vocabularies-7]: ../../design.md#7-every-file-of-the-folder-joins
