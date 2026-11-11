## REMOVED Requirements

### Requirement: Known gaps are listed apart from what differs by choice

**Reason**: OpenSpec cannot drop one scenario of a modified requirement, and the gap for plain text closes, so the requirement is removed and added again below without that scenario.

**Migration**: Read the requirement of the same body under the ADDED section, named "Known gaps are listed apart from the choices". Its other scenarios are unchanged.

## ADDED Requirements

### Requirement: Known gaps are listed apart from the choices

This spec SHALL list each thing Vale 3.26.0 does that hakkiri does not yet do, apart from the intended differences, with its queue row.  
A pull request that closes a gap SHALL remove its scenario, and record any difference the closing leaves as an intended difference.

#### Scenario: The built-in spelling rule

- **WHEN** a project enables Vale's `Vale.Spelling`, which ships with a default ignore list
- **THEN** hakkiri has no such rule, and a style names its own dictionary and lists (row [HKR-026][ledger], [spelling goals][spelling-goals])

#### Scenario: A YAML vocabulary

- **WHEN** a vocabulary folder holds `vocab.yml` with `accept` and `reject` entries, and options such as `regex`, `ignorecase` and `flags`
- **THEN** hakkiri reads `accept.txt` and `reject.txt` only, and Vale reads the YAML file too (row [HKR-041][ledger], [vocabularies 8][vocabularies-8])

#### Scenario: The Markdown parser

- **WHEN** a Markdown file holds syntax that hakkiri's parser and Vale's converter read differently
- **THEN** no such difference is measured yet, so the cases are unknown (row [HKR-027][ledger], [tree 1][tree-1])

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
- **THEN** hakkiri runs the rest of the style and fails the run for the rule until the config sets it `off`.
  It refuses the case field, where Vale selects by CSS over the rendered HTML (row [HKR-039][ledger], [test 2][test-2])

#### Scenario: A Vale configuration file

- **WHEN** a project holds a `.vale.ini` and runs `hakkiri import vale`
- **THEN** hakkiri has no such command yet, and the config is written by hand (row [HKR-035][ledger])

#### Scenario: An action

- **WHEN** a rule holds an `action`
- **THEN** hakkiri loads it and applies nothing, and Vale offers the fix (row [HKR-011][ledger])

### Requirement: Readers differ from Vale in what a file with no reader gets

hakkiri SHALL read a selected file that no reader reads as plain text, skip a binary file, and map a format as each scenario below says, where Vale 3.26.0 does otherwise.

#### Scenario: A manuscript

- **WHEN** a `.txt` file holds chapter lines
- **THEN** hakkiri reads paragraphs, and Vale reads a document whose chapter lines are headings ([plain-text 1][plain-text-1])

#### Scenario: A binary file

- **WHEN** a glob selects a file that holds a NUL byte
- **THEN** hakkiri skips it, and Vale has no such test and reads it as lines ([plain-text 5][plain-text-5])

#### Scenario: A plain-text file with markup

- **WHEN** `notes.cfg` holds a fenced block, a glob selects it and no `formats` entry maps it
- **THEN** hakkiri reads the fence as text, as Vale reads a file of lines ([plain-text 1][plain-text-1])

#### Scenario: A format in a package and in the project

- **WHEN** a package maps `txt: md` and the project maps `txt: text`
- **THEN** hakkiri reads the project's value, and Vale takes the first value for `[formats]`, so the package wins ([package settings 5][settings-5])

[engine-14]: ../../../archive/2026-11-10-rule-engine/design.md#14-formulas-run-on-an-own-parser
[html-7]: ../../../archive/2026-11-10-html-block-text/design.md#not-ported
[ledger]: ../../../../../tasks.yaml
[plain-text-1]: ../../design.md#1-plain-text-is-one-raw-root-of-paragraphs
[plain-text-5]: ../../design.md#5-a-binary-file-is-skipped
[settings-5]: ../../../archive/2026-11-11-package-settings/design.md#5-scalars-the-project-wins
[spelling-goals]: ../../../archive/2026-11-10-spelling-and-affix-checks/design.md#goals--non-goals
[styles-4]: ../../../archive/2026-11-11-vale-styles/design.md#4-a-rule-hakkiri-cannot-run-is-reported-apart
[test-2]: ../../../archive/2026-11-10-test-command/design.md#2-a-case-is-a-small-record
[tree-1]: ../../../archive/2026-11-10-document-tree/design.md#1-markdown-rs-parses-markdown-and-mdx
[vocabularies-8]: ../../../archive/2026-11-11-vocabularies/design.md#8-what-stays-different
