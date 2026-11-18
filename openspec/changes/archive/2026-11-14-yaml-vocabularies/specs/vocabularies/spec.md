## MODIFIED Requirements

### Requirement: A vocabulary is a folder of word lists in a package

hakkiri SHALL read a named vocabulary from `config/vocabularies/<name>/` under a loaded package folder.
It SHALL read every `accept.txt`, `reject.txt`, `vocab.yml` and `vocab.yaml` below that folder and no other file.

#### Scenario: A list in a subfolder

- **WHEN** the vocabulary `Team` contains `accept.txt` with `GitHub`, `sub/accept.txt` with `Kubernetes`, and `words.txt` with `Gamma`
- **THEN** `GitHub` and `Kubernetes` are accepted terms and `Gamma` is no term ([vocabularies 3][vocabularies-3])

#### Scenario: A vocabulary that is not there

- **WHEN** the config lists `Nope` and no loaded package contains `config/vocabularies/Nope`
- **THEN** hakkiri reports an error that shows `Nope` and the line of the key, and exits with a failure status

#### Scenario: Two packages hold one name

- **WHEN** two packages each contain `config/vocabularies/Team`
- **THEN** hakkiri reports an error that shows both folders, and exits with a failure status

#### Scenario: Text and YAML files together

- **WHEN** the vocabulary `Mixed` contains `accept.txt` with `Alpha`, `vocab.yml` that accepts `Beta` and `vocab.yaml` that accepts `Gamma`
- **THEN** `Alpha`, `Beta` and `Gamma` are accepted terms, and `Delta` draws its alert ([yaml-vocabularies 7][yaml-vocabularies-7])

## ADDED Requirements

### Requirement: A YAML entry is a term, or a term with options

A `vocab.yml` or `vocab.yaml` file SHALL have the keys `affix`, `accept` and `reject`.
An entry of `accept` or `reject` SHALL be a term, a term mapped to nothing, or a term mapped to the options `regex`, `ignorecase` and `flags`.
It SHALL give the patterns that a line of a text list would.

#### Scenario: A plain term

- **WHEN** `accept` contains `GitHub`, `Node.js` and `C++`
- **AND** a rule flags `GitHub`, `Node.js`, `Nodexjs`, `C++` and `Cxx`, and the text contains each
- **THEN** only `Nodexjs` and `Cxx` draw an alert, since a period is literal and `C++` is quoted ([yaml-vocabularies 3][yaml-vocabularies-3])

#### Scenario: A pattern, case and flags

- **WHEN** `accept` contains `harness(?:es)?` with `regex: true`, `zebra` with `ignorecase: true` and `utilize` with `flags: DSG`
- **THEN** `harness` and `harnesses` are accepted, so are `zebra`, `Zebra` and `ZEBRA`, and so are `utilize`, `utilizes`, `utilizing` and `utilized`, but `harnessed` and `utilization` are not

#### Scenario: A term mapped to nothing

- **WHEN** `accept` contains `Kubernetes:` with nothing after the colon
- **THEN** `Kubernetes` is accepted and `kubernetes` is not

#### Scenario: A number as a term

- **WHEN** `accept` contains `3.10` unquoted, and a rule flags `3.10` and `3.1`
- **THEN** only `3.1` draws an alert ([yaml-vocabularies 4][yaml-vocabularies-4])

#### Scenario: Rejected terms

- **WHEN** `reject` contains `ba+r` with `regex: true` and `Baz` with `ignorecase: true`, and the text is `baar and bar, BAZ and baz.`
- **THEN** `Vale.Avoid` reports `baar`, `bar`, `BAZ` and `baz`, each as ``Avoid using '<text>'.``

#### Scenario: A null entry and a YAML 1.1 word

- **WHEN** a list contains `-` alone, and an entry sets `regex: yes`
- **THEN** the empty entry adds nothing and `yes` turns the option on, as in Vale ([yaml-vocabularies 5][yaml-vocabularies-5])

### Requirement: A wrong YAML entry is refused with its file and line

hakkiri SHALL reject a vocabulary file that has a wrong entry or a missing affix file.
Each problem SHALL show the file and the line that Vale reports, and one run SHALL report every one.

#### Scenario: A wrong shape

- **WHEN** an entry has more than one key, a key mapped to a string or a list, or an unknown option
- **OR** it has a `regex` or `ignorecase` that is no boolean, or an empty term
- **THEN** hakkiri rejects the file

#### Scenario: A wrong value

- **WHEN** an entry has both `regex` and `flags`, a pattern that does not compile, flags that do not expand, or an `affix` that refers to a missing file
- **THEN** hakkiri rejects the file

#### Scenario: Each fault

- **WHEN** a vocabulary contains only one of those faults, in `two-keys`, `scalar-value`, `list-value`, `unknown-option`, `regex-and-flags`, `bad-regex`, `bad-flag`, `empty-term`, `missing-affix`, `bad-bool` or `entry-list` of the fixture
- **THEN** hakkiri exits 2 with ``failed <file>: line N:`` where N is the line of Vale's error

#### Scenario: Several faults in one file

- **WHEN** a file has an empty term on line 2 and an unknown option on line 4
- **AND** it has a pattern that does not compile on line 5
- **THEN** hakkiri reports all three problems, each with its line, where Vale reports the first ([yaml-vocabularies 2][yaml-vocabularies-2])

### Requirement: The affix file comes from the packages

`affix` SHALL refer to a file `<name>.aff` in the `config/dictionaries` folder of the package that contains the vocabulary.
After that package, hakkiri SHALL look in every other loaded package in order.
No name, and `en_US-web` with no such file, SHALL read the built-in dictionary.
hakkiri SHALL reject a name that is not one file name.

#### Scenario: The holder first

- **WHEN** two packages each contain `config/dictionaries/custom.aff`, with the suffixes `zz` and `qq`
- **AND** the package with `zz` contains the vocabulary that sets `affix: custom`
- **THEN** `foo` and `foozz` are accepted and `fooqq` is not, whatever the order of the packages

#### Scenario: Another package

- **WHEN** only the other package contains `custom.aff`, with the suffix `qq`
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
