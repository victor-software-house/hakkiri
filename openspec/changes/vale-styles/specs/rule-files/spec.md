## REMOVED Requirements

### Requirement: A script rule is refused

**Reason**: A script rule no longer refuses its whole style. The loader reports it apart and the rest of the style runs, so the requirement is removed and added again below under a name that tells it.

**Migration**: Read "A script rule is reported, and the rest of its style runs".

## ADDED Requirements

### Requirement: A script rule is reported, and the rest of its style runs

hakkiri SHALL NOT run a rule whose `extends` is `script`.  
It SHALL report it as a rule it cannot run, with an error that names the rule file, as the project-config spec fixes.

#### Scenario: A style ships a script rule

- **WHEN** a style folder `Team` holds `Grade.yml` with `extends: script` and a glob lists `Team`
- **THEN** hakkiri reports an error naming `Team/Grade.yml` and exits 2, and the other rules of `Team` run
