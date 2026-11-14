## ADDED Requirements

### Requirement: A rule with forms is read by hakkiri alone

hakkiri SHALL read a `sequence` rule that sets `forms`, which Vale 3.26.0 does not define.

#### Scenario: A rule of forms with no tokens

- **WHEN** a `sequence` rule sets `forms` and no `tokens`
- **THEN** hakkiri reads it, and Vale stops with a panic at its type assertion on `tokens` ([sequence-forms 4][sequence-forms-4])

[sequence-forms-4]: ../../design.md#4-a-rule-file-with-forms-is-read-by-hakkiri-alone
