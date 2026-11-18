## ADDED Requirements

### Requirement: The config names the vocabularies a project uses

The project config SHALL list vocabularies by folder name in a `vocab` list, at its top level and in a glob of `files`.  
hakkiri SHALL read each from a loaded package before it builds any rule.

#### Scenario: A vocabulary of a package

- **WHEN** `packages` contains `{path: styles}`, `styles/config/vocabularies/Team/accept.txt` contains `GitHub`, and the top level contains `vocab: [Team]`
- **THEN** `GitHub` is an accepted term of every rule that reads one

#### Scenario: A glob's vocabulary

- **WHEN** `files` contains `'docs/*.md': {vocab: [Docs]}` and `Docs` accepts `Zebra`
- **THEN** `Zebra` is accepted in the files that glob matches, and in no other

#### Scenario: No vocab key

- **WHEN** the config contains no `vocab` key
- **THEN** no accepted or rejected term exists, and every rule drops what it dropped before

#### Scenario: A vocabulary of a fetched package

- **WHEN** a package pinned by `url` and `sha256` contains `styles/config/vocabularies/Team/`, and `vocab` names `Team`
- **THEN** hakkiri reads the lists from the cache folder of that package, and a second run makes no request

#### Scenario: A vocab list that is not a list

- **WHEN** `vocab` contains the word `Team` and not a list
- **THEN** hakkiri reports an error on the line of `vocab`, and exits with a failure status
