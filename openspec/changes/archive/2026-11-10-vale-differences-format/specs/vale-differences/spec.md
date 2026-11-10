## ADDED Requirements

### Requirement: A comment reads as Markdown

hakkiri SHALL read every comment of a code file as Markdown, where Vale 3.26.0 reads it as plain lines unless a View names a markup format.

#### Scenario: A code span in a plain comment

- **WHEN** a Rust file holds ``// Use `utilize` here.`` and an `existence` rule matches `utilize`
- **THEN** hakkiri reads the span as code and reports nothing, and Vale reports `utilize` inside the backticks ([format 8][format-8])

### Requirement: Rule files differ from Vale in what they accept or select

hakkiri SHALL load, refuse or select as each scenario below says, where Vale 3.26.0 does otherwise.

#### Scenario: A table row is a scope

- **WHEN** a rule has `scope: table.row` and a row holds the matched text
- **THEN** hakkiri selects the text of the row, and Vale loads the rule and selects nothing, since no block has a `row` part ([format 9][format-9])

#### Scenario: A scope no node holds

- **WHEN** a rule has `scope: nonsense`
- **THEN** hakkiri refuses it as an unknown scope, and Vale loads it and selects nothing ([engine 3][engine-3])

#### Scenario: A metric counts tokens

- **WHEN** a `metric` rule sets `tokens: [";"]` and the formula `occurrences / words * 1000`
- **THEN** hakkiri measures the rate of the matches, and Vale refuses `tokens` as an invalid key and `occurrences` as an unresolved reference ([format 4][format-4], [engine 11][engine-11])

#### Scenario: A formula outside the listed syntax

- **WHEN** a `metric` rule has the formula `words > 1 ? 7.0 : 0.0`
- **THEN** hakkiri refuses it, since it reads numbers, variables, operators, `math` functions and `round`, and Vale evaluates one Tengo expression and reports 7.00 ([engine 14][engine-14])

#### Scenario: A script rule

- **WHEN** a rule has `extends: script`
- **THEN** hakkiri refuses it, and Vale runs its Tengo program ([format 5][format-5])

### Requirement: Configuration and packages are hakkiri's own

hakkiri SHALL read its settings from `.ctl/hakkiri.yaml` and take Vale styles only through `hakkiri import vale`, where Vale reads `.vale.ini`.

#### Scenario: A Vale configuration

- **WHEN** a project holds a `.vale.ini` and no `.ctl/hakkiri.yaml`
- **THEN** hakkiri reads no setting from it, and Vale reads it ([bootstrap 1][bootstrap-1])

#### Scenario: A rule is turned off

- **WHEN** a glob in the project config turns a rule off
- **THEN** it sets the rule to `off`, where Vale writes `NO` ([format 6][format-6])

#### Scenario: A package is fetched on use

- **WHEN** a project pins a package by version and hash
- **THEN** hakkiri fetches and caches it with no separate sync step, where Vale needs `vale sync` ([bootstrap 5][bootstrap-5])

[format-4]: ../../../2026-11-10-rule-format/design.md#4-a-rate-is-a-metric-that-counts-tokens
[format-5]: ../../../2026-11-10-rule-format/design.md#5-release-1-implements-every-vale-check-type-but-script
[format-6]: ../../../2026-11-10-rule-format/design.md#6-the-project-config-uses-lowercase-keys
[format-8]: ../../../2026-11-10-rule-format/design.md#8-every-comment-reads-as-markdown
[format-9]: ../../../2026-11-10-rule-format/design.md#9-a-table-row-is-a-scope
[engine-3]: ../../../2026-11-10-rule-engine/design.md#3-a-scope-part-matches-by-dotted-prefix
[engine-11]: ../../../2026-11-10-rule-engine/design.md#11-metric-tokens-match-as-existence-tokens
[engine-14]: ../../../2026-11-10-rule-engine/design.md#14-formulas-run-on-an-own-parser
[bootstrap-1]: ../../../2026-11-09-bootstrap-hakkiri/design.md#1-own-format-built-from-the-best-prior-art
[bootstrap-5]: ../../../2026-11-09-bootstrap-hakkiri/design.md#5-rules-ship-as-versioned-packages
