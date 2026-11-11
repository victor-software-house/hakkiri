## ADDED Requirements

### Requirement: The test command differs from Vale's

hakkiri SHALL read cases and print their report as each scenario below says, where `vale test` in Vale 3.26.0 does otherwise.

#### Scenario: A View

- **WHEN** a case sets `view`
- **THEN** hakkiri refuses the field as unknown, and Vale reads the View under its search path ([test 2][test-2])

#### Scenario: A `.yaml` file

- **WHEN** a directory holds `Rule.yaml` with `extends` and `tests`
- **THEN** hakkiri reads no case from it, and Vale reads its cases ([test 5][test-5])

#### Scenario: The report

- **WHEN** a case fails
- **THEN** hakkiri prints its file, name, reason and alerts in its own layout, and Vale prints a marked heading and a coloured diff ([test 7][test-7])

#### Scenario: A name two rule files share that no case flags

- **WHEN** `Terms.yml` and `Terms.custom.yml` both give `Team.Terms`, and `test --coverage` finds that no case flags it
- **THEN** hakkiri lists it once with both files, and Vale lists `Team.Terms` twice and names no file ([rule-names 4][rule-names-4])

[test-2]: ../../design.md#2-a-case-is-a-small-record
[test-5]: ../../design.md#5-finding-cases
[test-7]: ../../design.md#7-the-report-and-the-exit-status
[rule-names-4]: ../../../rule-names/design.md#4-test-finds-the-style-folder-as-vale-does
