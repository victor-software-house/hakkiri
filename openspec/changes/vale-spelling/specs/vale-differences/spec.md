## REMOVED Requirements

### Requirement: The built-in spelling rule is a known gap

**Reason**: The style `Vale` now holds `Vale.Spelling`, so the gap closes.

**Migration**: Read the requirement "The built-in spelling rule differs from Vale's in these ways" under the ADDED section for what still differs.

## ADDED Requirements

### Requirement: The built-in spelling rule differs from Vale's in these ways

hakkiri SHALL run `Vale.Spelling` as Vale does, apart from the scenarios below and the differences of a `spelling` rule that this spec already lists.

#### Scenario: Several packages

- **WHEN** a project loads two packages that each hold a `config/ignore` folder or a `config/dictionaries` folder
- **THEN** hakkiri reads both, in order, and Vale reads one styles path ([vale-spelling 2][vale-spelling-2])

#### Scenario: A code span in a code comment

- **WHEN** a Rust comment holds `` `blorftCode` `` and `custom` turns the built-in filters off
- **THEN** hakkiri reads no word in the code span, and Vale reports the span with its backticks as one word ([vale-spelling 5][vale-spelling-5])

#### Scenario: The name after a param tag

- **WHEN** a TypeScript comment holds `@param nme the nme of it`
- **THEN** hakkiri reads no word for the name `nme`, and Vale reports it, and reports `@param` when `custom` turns the filters off ([vale-spelling 5][vale-spelling-5])

[vale-spelling-2]: ../../design.md#2-the-built-in-rule-reads-the-settings-of-every-package
[vale-spelling-5]: ../../design.md#5-what-the-probe-shows-and-hakkiri-differs-on
