## MODIFIED Requirements

### Requirement: The built-in style Vale contains the vocabulary rules

hakkiri SHALL accept `Vale` in a glob's `styles` as a style whose rules are `Repetition` and `Spelling`.  
The style SHALL also have `Terms` and `Avoid` when a named vocabulary at the top level has a term.  
The style SHALL exist whatever the vocabularies contain.

#### Scenario: A style named Vale

- **WHEN** a glob lists `styles: [Vale]` and the config names a vocabulary with an accepted term
- **THEN** the glob runs `Vale.Repetition`, `Vale.Spelling` and `Vale.Terms`

#### Scenario: No vocabulary

- **WHEN** a glob lists `styles: [Vale]` and no named vocabulary has a term
- **THEN** the glob runs `Vale.Repetition` and `Vale.Spelling`
