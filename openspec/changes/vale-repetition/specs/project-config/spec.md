## MODIFIED Requirements

### Requirement: The built-in style Vale holds the vocabulary rules

hakkiri SHALL accept `Vale` in a glob's `styles` as a style whose rules are `Repetition`, and `Terms` and `Avoid` when a named vocabulary at the top level holds a term.
The style SHALL exist whatever the vocabularies hold.

#### Scenario: A style named Vale

- **WHEN** a glob lists `styles: [Vale]` and the config names a vocabulary with an accepted term
- **THEN** the glob runs `Vale.Repetition` and `Vale.Terms`

#### Scenario: No vocabulary

- **WHEN** a glob lists `styles: [Vale]` and no named vocabulary holds a term
- **THEN** the glob runs `Vale.Repetition` alone
