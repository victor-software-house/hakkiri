## ADDED Requirements

### Requirement: The built-in style Vale contains the vocabulary rules

hakkiri SHALL accept `Vale` in a glob's `styles` as a style whose rules are `Terms` and `Avoid`, when a listed vocabulary contains a term.  
When none does, hakkiri SHALL reject it as a style that no package contains.

#### Scenario: A style named Vale

- **WHEN** a glob lists `styles: [Vale]` and the config names a vocabulary with an accepted term
- **THEN** the glob runs `Vale.Terms`

#### Scenario: No vocabulary

- **WHEN** a glob lists `styles: [Vale]` and no named vocabulary contains a term
- **THEN** hakkiri reports that `Vale` is not a style in any package, and exits with a failure status
