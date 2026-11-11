## ADDED Requirements

### Requirement: The cases of a rule hakkiri cannot run are not run

`hakkiri test` SHALL report the cases of a rule hakkiri cannot run as not run, with a failure that says its cases did not run.  
It SHALL report a case that runs through a config as not run when the config runs such a rule on the case's file.

#### Scenario: A script rule with a case

- **WHEN** `Cannot.yml` holds `extends: script` and one case, and `Parent.yml` holds a case that passes
- **THEN** hakkiri prints a failure that says its cases did not run, counts the passing case, and exits 2

#### Scenario: A case through a config that runs a rule hakkiri cannot run

- **WHEN** a test file holds a case that runs through a config whose listed style holds a script rule
- **THEN** hakkiri reports the case as not run, naming the rule, and exits 2
