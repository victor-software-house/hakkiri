## ADDED Requirements

### Requirement: The config names the vocabularies a project uses

The project config SHALL name vocabularies by the name of their folder, and hakkiri SHALL read each from a loaded package before it builds any rule.
The key and its place in the config wait for the operator's choice (design decision 2), so these scenarios say "the vocabulary key".

#### Scenario: A vocabulary of a package

- **WHEN** `packages` holds `{path: styles}`, `styles/config/vocabularies/Team/accept.txt` holds `GitHub`, and the vocabulary key names `Team`
- **THEN** `GitHub` is an accepted term of every rule that reads one

#### Scenario: No vocabulary key

- **WHEN** the config holds no vocabulary key
- **THEN** no accepted or rejected term exists, and every rule holds back what it held back before

#### Scenario: A vocabulary of a fetched package

- **WHEN** a package pinned by `url` and `sha256` holds `styles/config/vocabularies/Team/`, and the vocabulary key names `Team`
- **THEN** hakkiri reads the lists from the cache folder of that package, and a second run asks for nothing
