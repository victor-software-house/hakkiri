## MODIFIED Requirements

### Requirement: Known gaps are listed apart from the intended differences

This spec SHALL list each thing Vale 3.26.0 does that hakkiri does not yet do, apart from the intended differences, with its queue row.  
A pull request that closes a gap SHALL remove its scenario, and record any difference the closing leaves as an intended difference.

#### Scenario: The built-in spelling rule

- **WHEN** a project enables Vale's `Vale.Spelling`, which ships with a default ignore list
- **THEN** hakkiri has no such rule, and a style names its own dictionary and lists (row [HKR-026][ledger], [spelling goals][spelling-goals])

#### Scenario: Vocabularies

- **WHEN** a section of a Vale config sets `Vocab`, or the top level names a vocabulary
- **THEN** hakkiri holds back the matches the accepted terms name, and builds no `Vale.Terms`, `Vale.Avoid`, `Vale.<Name>.Terms` or `Vale.<Name>.Avoid`, which Vale runs for the project or the section (row [HKR-042][ledger], [vocabulary-rules 1][vocabulary-rules-1])

#### Scenario: A YAML vocabulary

- **WHEN** a vocabulary folder holds `vocab.yml` with `accept` and `reject` entries, and options such as `regex`, `ignorecase` and `flags`
- **THEN** hakkiri reads `accept.txt` and `reject.txt` only, and Vale reads the YAML file too (row [HKR-041][ledger], [vocabularies 8][vocabularies-8])

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
- **THEN** hakkiri runs the rest of the style and fails the run for the rule until the config sets it `off`.
  It refuses the case field, where Vale selects by CSS over the rendered HTML (row [HKR-039][ledger], [test 2][test-2])

#### Scenario: A Vale configuration file

- **WHEN** a project holds a `.vale.ini` and runs `hakkiri import vale`
- **THEN** hakkiri has no such command yet, and the config is written by hand (row [HKR-035][ledger])

#### Scenario: An action

- **WHEN** a rule holds an `action`
- **THEN** hakkiri loads it and applies nothing, and Vale offers the fix (row [HKR-011][ledger])

## ADDED Requirements

### Requirement: Vocabularies differ from Vale's in these ways

hakkiri SHALL read vocabularies as Vale does, apart from the scenarios below.

#### Scenario: A term that is not a pattern

- **WHEN** a list holds a term such as `C(` that is not a valid pattern
- **THEN** hakkiri reports an error that names the file and line, where Vale stops with a pattern error that has no line ([vocabularies 4][vocabularies-4])

#### Scenario: A term that Vale's engines refuse

- **WHEN** a list holds `C++`
- **THEN** hakkiri reads it as the pattern `C` with a possessive plus, and holds back `C` and `CC`, where Vale stops with a pattern error ([bootstrap 8][bootstrap-8])

#### Scenario: Two packages with one vocabulary

- **WHEN** two packages each hold a vocabulary `Team`
- **THEN** hakkiri refuses the config and names both, where Vale takes the first search path, or the last package after `vale sync` ([vocabularies 3][vocabularies-3])

#### Scenario: The per-user styles folder

- **WHEN** a vocabulary is in no package of the project
- **THEN** hakkiri reports it unknown, where Vale also searches its per-user styles folder ([vocabularies 3][vocabularies-3])

#### Scenario: An unknown vocabulary

- **WHEN** the config names a vocabulary that no package holds
- **THEN** hakkiri names the vocabulary and the line of the key, where Vale lists the search paths and gives no line ([vocabularies 3][vocabularies-3])

[vocabularies-3]: ../../design.md#3-a-vocabulary-is-a-folder-in-a-loaded-package
[vocabularies-4]: ../../design.md#4-a-line-of-a-list-is-a-pattern
[vocabularies-8]: ../../design.md#8-what-stays-different
[vocabulary-rules-1]: ../../../2026-11-11-vocabulary-rules/design.md#1-terms-and-avoid-are-rules-of-a-style-named-vale
[ledger]: ../../../../../../tasks.yaml
[spelling-goals]: ../../../2026-11-10-spelling-and-affix-checks/design.md#goals--non-goals
[tree-1]: ../../../2026-11-10-document-tree/design.md#1-markdown-rs-parses-markdown-and-mdx
[check-5]: ../../../2026-11-10-check-command/design.md#5-a-language-is-read-by-extension
[test-2]: ../../../2026-11-10-test-command/design.md#2-a-case-is-a-small-record
[bootstrap-8]: ../../../2026-11-09-bootstrap-hakkiri/design.md#8-two-regex-engines-with-a-limit
