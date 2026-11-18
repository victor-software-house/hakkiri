## ADDED Requirements

### Requirement: The cases of a rule hakkiri cannot run are not run

`hakkiri test` SHALL report the cases of a rule hakkiri cannot run as not run, with a failure that states its cases did not run.  
It SHALL report a case that runs through a config as not run when the config runs such a rule on the case's file.

#### Scenario: A script rule with a case

- **WHEN** `Cannot.yml` contains `extends: script` and one case, and `Parent.yml` contains a case that passes
- **THEN** hakkiri prints a failure that states its cases did not run, counts the passing case, and exits 2

#### Scenario: A case through a config that runs a rule hakkiri cannot run

- **WHEN** a test file contains a case that runs through a config whose listed style contains a script rule
- **THEN** hakkiri reports the case as not run, with the rule, and exits 2
