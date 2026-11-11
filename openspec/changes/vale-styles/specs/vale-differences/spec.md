## REMOVED Requirements

### Requirement: Rule files differ from Vale in what they accept or select

**Reason**: Two of its scenarios, a formula outside the listed syntax and a script rule, are things Vale runs and hakkiri does not, so they move to the Known gaps with a row each. OpenSpec cannot drop a scenario of a modified requirement, so the requirement is removed and added again below with the three scenarios that remain.

**Migration**: Read "Rule files differ from Vale in what they select". The formula and the script scenarios are under the Known gaps.

### Requirement: Configuration and packages are hakkiri's own

**Reason**: It states that Vale styles enter only through `hakkiri import vale`, which is no longer so: hakkiri reads a Vale style folder as it is. The requirement is removed and added again below with the new text.

**Migration**: Read "Configuration is hakkiri's own, and Vale styles load as they are".

## ADDED Requirements

### Requirement: Rule files differ from Vale in what they select

hakkiri SHALL load or select as each scenario below says, where Vale 3.26.0 does otherwise.

#### Scenario: A table row is a scope

- **WHEN** a rule has `scope: table.row` and a row holds the matched text
- **THEN** hakkiri selects the text of the row, and Vale loads the rule and selects nothing, since no block has a `row` part ([format 9][format-9])

#### Scenario: A scope no node holds

- **WHEN** a rule has `scope: nonsense`
- **THEN** hakkiri refuses it as an unknown scope, and Vale loads it and selects nothing ([engine 3][engine-3])

#### Scenario: A metric counts tokens

- **WHEN** a `metric` rule sets `tokens: [";"]` and the formula `occurrences / words * 1000`
- **THEN** hakkiri measures the rate of the matches, and Vale refuses `tokens` as an invalid key and `occurrences` as an unresolved reference ([format 4][format-4], [engine 11][engine-11])

### Requirement: Configuration is hakkiri's own, and Vale styles load as they are

hakkiri SHALL read its settings from `.ctl/hakkiri.yaml`, where Vale reads `.vale.ini`, and SHALL load a Vale style folder as it is.

#### Scenario: A Vale configuration

- **WHEN** a project holds a `.vale.ini` and no `.ctl/hakkiri.yaml`
- **THEN** hakkiri reads no setting from it, and Vale reads it ([bootstrap 1][bootstrap-1], row [HKR-035][ledger])

#### Scenario: A rule is turned off

- **WHEN** a glob in the project config turns a rule off
- **THEN** it sets the rule to `off`, where Vale writes `NO` ([format 6][format-6])

#### Scenario: A package is fetched on use

- **WHEN** a project pins a package by version and hash
- **THEN** hakkiri fetches and caches it with no separate sync step, where Vale needs `vale sync` ([bootstrap 5][bootstrap-5])

#### Scenario: A Vale style folder

- **WHEN** a package holds a Vale style folder, such as Voices, Std or Readability
- **THEN** hakkiri loads it with no conversion, as Vale does, apart from the rules the Known gaps list ([styles 1][styles-1])

## MODIFIED Requirements

### Requirement: Known gaps are listed apart from the intended differences

This spec SHALL list each thing Vale 3.26.0 does that hakkiri does not yet do, apart from the intended differences, with its queue row.  
A pull request that closes a gap SHALL remove its scenario, and record any difference the closing leaves as an intended difference.

#### Scenario: The built-in spelling rule

- **WHEN** a project enables Vale's `Vale.Spelling`, which ships with a default ignore list
- **THEN** hakkiri has no such rule, and a style names its own dictionary and lists (row [HKR-026][ledger], [spelling goals][spelling-goals])

#### Scenario: Vocabularies

- **WHEN** a rule sets `vocab`, or a project lists accepted and rejected terms
- **THEN** hakkiri loads the field and reads no vocabulary, and Vale adds the accepted terms to the exceptions and generates the `Vale.Terms` and `Vale.Avoid` rules (row [HKR-019][ledger], [case 1][case-1])

#### Scenario: The Markdown parser

- **WHEN** a Markdown file holds syntax that hakkiri's parser and Vale's converter read differently
- **THEN** no such difference is measured yet, so the cases are unknown (row [HKR-027][ledger], [tree 1][tree-1])

#### Scenario: A plain-text file

- **WHEN** a glob matches a file that no language reads, such as `notes.txt`
- **THEN** hakkiri skips it in a walk and refuses it when named, and Vale lints it as plain text (row [HKR-030][ledger], [check 5][check-5])

#### Scenario: Comments that turn rules off and on

- **WHEN** a file holds a `vale off` comment, a `vale on` comment or a comment that turns one rule off
- **THEN** hakkiri lints the text after it as before, and Vale skips it until the matching comment (row [HKR-031][ledger], [html 7][html-7])

#### Scenario: A script rule

- **WHEN** a rule has `extends: script`
- **THEN** hakkiri runs the rest of its style and fails the run for the rule until the config sets it `off`, and Vale runs its Tengo program (row [HKR-036][ledger], [styles 4][styles-4])

#### Scenario: A formula outside the listed syntax

- **WHEN** a `metric` rule has the formula `words > 1 ? 7.0 : 0.0`
- **THEN** hakkiri refuses it, since it reads numbers, variables, operators, `math` functions and `round`, and Vale evaluates one Tengo expression and reports 7.00 (row [HKR-037][ledger], [engine 14][engine-14])

#### Scenario: The built-in repetition rule

- **WHEN** a glob lists the style `Vale`, which holds `Vale.Repetition`
- **THEN** hakkiri has no such style and refuses the name, and Vale reports a repeated word (row [HKR-038][ledger])

#### Scenario: A doc selector or a View

- **WHEN** a rule has `scope: doc(h1)`, a section names a View, or a case sets `view`
- **THEN** hakkiri runs the rest of the style and fails the run for the rule until the config sets it `off`, refuses the case field, and Vale selects by CSS over the rendered HTML (row [HKR-039][ledger], [test 2][test-2])

#### Scenario: A Vale configuration file

- **WHEN** a project holds a `.vale.ini` and runs `hakkiri import vale`
- **THEN** hakkiri has no such command yet, and the config is written by hand (row [HKR-035][ledger])

#### Scenario: An action

- **WHEN** a rule holds an `action`
- **THEN** hakkiri loads it and applies nothing, and Vale offers the fix (row [HKR-011][ledger])

[ledger]: ../../../../../tasks.yaml
[bootstrap-1]: ../../../archive/2026-11-09-bootstrap-hakkiri/design.md#1-own-format-built-from-the-best-prior-art
[bootstrap-5]: ../../../archive/2026-11-09-bootstrap-hakkiri/design.md#5-rules-ship-as-versioned-packages
[case-1]: ../../../archive/2026-11-10-case-and-sequence-checks/design.md#1-case-conversion-ports-proses-converters
[check-5]: ../../../archive/2026-11-10-check-command/design.md#5-a-language-is-read-by-extension
[engine-3]: ../../../archive/2026-11-10-rule-engine/design.md#3-a-scope-part-matches-by-dotted-prefix
[engine-11]: ../../../archive/2026-11-10-rule-engine/design.md#11-metric-tokens-match-as-existence-tokens
[engine-14]: ../../../archive/2026-11-10-rule-engine/design.md#14-formulas-run-on-an-own-parser
[format-4]: ../../../archive/2026-11-10-rule-format/design.md#4-a-rate-is-a-metric-that-counts-tokens
[format-6]: ../../../archive/2026-11-10-rule-format/design.md#6-the-project-config-uses-lowercase-keys
[format-9]: ../../../archive/2026-11-10-rule-format/design.md#9-a-table-row-is-a-scope
[html-7]: ../../../archive/2026-11-10-html-block-text/design.md#not-ported
[spelling-goals]: ../../../archive/2026-11-10-spelling-and-affix-checks/design.md#goals--non-goals
[styles-1]: ../../design.md#1-the-loader-resolves-inheritance
[styles-4]: ../../design.md#4-a-rule-hakkiri-cannot-run-is-reported-apart
[test-2]: ../../../archive/2026-11-10-test-command/design.md#2-a-case-is-a-small-record
[tree-1]: ../../../archive/2026-11-10-document-tree/design.md#1-markdown-rs-parses-markdown-and-mdx
