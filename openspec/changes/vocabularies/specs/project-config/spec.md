## ADDED Requirements

### Requirement: The config names the vocabularies a project uses

The project config SHALL name vocabularies by the name of their folder in a `vocab` list, at its top level and in a glob of `files`, and hakkiri SHALL read each from a loaded package before it builds any rule.

#### Scenario: A vocabulary of a package

- **WHEN** `packages` holds `{path: styles}`, `styles/config/vocabularies/Team/accept.txt` holds `GitHub`, and the top level holds `vocab: [Team]`
- **THEN** `GitHub` is an accepted term of every rule that reads one

#### Scenario: A glob's vocabulary

- **WHEN** `files` holds `'docs/*.md': {vocab: [Docs]}` and `Docs` accepts `Zebra`
- **THEN** `Zebra` is accepted in the files that glob matches, and in no other

#### Scenario: No vocab key

- **WHEN** the config holds no `vocab` key
- **THEN** no accepted or rejected term exists, and every rule holds back what it held back before

#### Scenario: A vocabulary of a fetched package

- **WHEN** a package pinned by `url` and `sha256` holds `styles/config/vocabularies/Team/`, and `vocab` names `Team`
- **THEN** hakkiri reads the lists from the cache folder of that package, and a second run makes no request

#### Scenario: A vocab list that is not a list

- **WHEN** `vocab` holds the word `Team` and not a list
- **THEN** hakkiri reports an error that names `vocab` and its line, and exits with a failure status
