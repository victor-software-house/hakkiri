## REMOVED Requirements

### Requirement: Known gaps are listed apart from what differs by choice

**Reason**: OpenSpec cannot drop one scenario of a modified requirement, and the gap for the repetition rule closes, so the requirement is removed and added again below without that scenario.

**Migration**: Read the requirement of the same body under the ADDED section, named "Known gaps are listed apart from the settled differences". Its other scenarios are unchanged.

### Requirement: Vocabulary rules differ from Vale's in these ways

**Reason**: The style `Vale` now exists with no term, so the scenario for it closes, and a scenario cannot be dropped alone.

**Migration**: Read the requirement under the ADDED section, named "Vocabulary rules differ from Vale's in this way".

## ADDED Requirements

### Requirement: Known gaps are listed apart from the settled differences

This spec SHALL list each thing Vale 3.26.0 does that hakkiri does not yet do, apart from the intended differences, with its queue row.  
A pull request that closes a gap SHALL remove its scenario, and record any difference the closing leaves as an intended difference.

#### Scenario: The built-in spelling rule

- **WHEN** a project enables Vale's `Vale.Spelling`, which ships with a default ignore list
- **THEN** hakkiri has no such rule, and a style names its own dictionary and lists (row [HKR-026][ledger], [spelling goals][spelling-goals])

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

#### Scenario: A doc selector or a View

- **WHEN** a rule has `scope: doc(h1)`, a section names a View, or a case sets `view`
- **THEN** hakkiri runs the rest of the style and fails the run for the rule until the config sets it `off`.
  It refuses the case field, where Vale selects by CSS over the rendered HTML (row [HKR-039][ledger], [test 2][test-2])

#### Scenario: A Vale configuration file

- **WHEN** a project holds a `.vale.ini` and runs `hakkiri import vale`
- **THEN** hakkiri has no such command yet, and the config is written by hand (row [HKR-035][ledger])

#### Scenario: An action

- **WHEN** a rule holds an `action`
- **THEN** hakkiri loads it and applies nothing, and Vale offers the fix (row [HKR-011][ledger])

### Requirement: Vocabulary rules differ from Vale's in this way

hakkiri SHALL build and run the rules of a vocabulary as Vale does, apart from the scenario below.

#### Scenario: An escape in a term

- **WHEN** `data\Wset` is accepted and the text is `A data set, data-set, dataxset and DATA SET.`
- **THEN** hakkiri flags `DATA SET`, since it lowercases the literal text of the term and keeps `\W`, and Vale flags `dataxset`, since it lowercases the escape to `\w` ([vocabulary-rules 1][vocabulary-rules-1])

[check-5]: ../../../archive/2026-11-10-check-command/design.md#5-a-language-is-read-by-extension
[engine-14]: ../../../archive/2026-11-10-rule-engine/design.md#14-formulas-run-on-an-own-parser
[html-7]: ../../../archive/2026-11-10-html-block-text/design.md#not-ported
[ledger]: ../../../../../tasks.yaml
[spelling-goals]: ../../../archive/2026-11-10-spelling-and-affix-checks/design.md#goals--non-goals
[styles-4]: ../../../archive/2026-11-11-vale-styles/design.md#4-a-rule-hakkiri-cannot-run-is-reported-apart
[test-2]: ../../../archive/2026-11-10-test-command/design.md#2-a-case-is-a-small-record
[tree-1]: ../../../archive/2026-11-10-document-tree/design.md#1-markdown-rs-parses-markdown-and-mdx
[vocabulary-rules-1]: ../../../archive/2026-11-11-vocabulary-rules/design.md#1-terms-and-avoid-are-rules-of-a-style-named-vale
