## ADDED Requirements

### Requirement: Rule names differ from Vale in when a fault is refused

hakkiri SHALL refuse a name clash or a bracket in a rule name when it loads the style folder, where Vale 3.26.0 refuses it only once a glob enables the style.

#### Scenario: A clash or a bracket in a style no glob enables

- **WHEN** a style folder holds `Terms.yml` and `Terms.custom.yml`, or `Weird[max].yml`, and no glob enables the style
- **THEN** hakkiri refuses the folder, since it loads every style of a package, and Vale lints with no error and refuses the same folder once its `BasedOnStyles` names it ([rule-names 3][rule-names-3])

[rule-names-3]: ../../design.md#3-two-files-with-one-name-are-refused
