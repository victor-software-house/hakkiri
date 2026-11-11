## REMOVED Requirements

### Requirement: Known gaps are listed apart from the differences

**Reason**: OpenSpec cannot drop one scenario of a modified requirement, and the gap for a rule setting that enables one rule closes, so the requirement is removed and added again below without that scenario.

**Migration**: Read the requirement of the same body under the ADDED section, named "Known gaps are listed apart from the intended differences". Its five remaining scenarios are unchanged.

## ADDED Requirements

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

### Requirement: A rule setting is spelled as hakkiri's config spells it

hakkiri SHALL write `on` and `off` where Vale 3.26.0 writes `YES` and `NO`, and SHALL refuse a rule key that names no rule and a value that is none of those words or a level.

#### Scenario: A rule turned on at its own level

- **WHEN** a glob turns a rule on without naming a level
- **THEN** it sets the rule to `on`, where Vale writes `YES` ([rule-setting 7][rule-setting-7])

#### Scenario: A key that names no rule

- **WHEN** a glob holds `Team.Nope: warning` and no loaded style holds a rule `Team.Nope`
- **THEN** hakkiri refuses it with the line of the key, and Vale ignores `Team.Nope = YES` ([rule-setting 7][rule-setting-7])

#### Scenario: A word that is no level

- **WHEN** a glob sets a rule to `yes`
- **THEN** hakkiri refuses it with the line, and Vale warns that `yes` is not `YES`, `NO`, `UNSET` or a level, and switches the rule off ([rule-setting 7][rule-setting-7])

[ledger]: ../../../../../tasks.yaml
[spelling-goals]: ../../../archive/2026-11-10-spelling-and-affix-checks/design.md#goals--non-goals
[case-1]: ../../../archive/2026-11-10-case-and-sequence-checks/design.md#1-case-conversion-ports-proses-converters
[tree-1]: ../../../archive/2026-11-10-document-tree/design.md#1-markdown-rs-parses-markdown-and-mdx
[check-5]: ../../../archive/2026-11-10-check-command/design.md#5-a-language-is-read-by-extension
[html-7]: ../../../archive/2026-11-10-html-block-text/design.md#not-ported
[rule-setting-7]: ../../design.md#7-what-stays-different
