## ADDED Requirements

### Requirement: The command line differs from Vale's

hakkiri SHALL print, report and exit as each scenario below says, where Vale 3.26.0 does otherwise.

#### Scenario: A line carries its level

- **WHEN** a Markdown file draws one warning
- **THEN** hakkiri prints `path:line:column: level Rule: message` and a count line, and Vale's `--output=line` prints `path:line:col:Rule:message` with no level ([check 7][check-7])

#### Scenario: The JSON is an object

- **WHEN** `--format json` is given
- **THEN** hakkiri prints one object holding `alerts`, `failures` and `files`, and Vale prints a map from each path to its alerts ([check 7][check-7])

#### Scenario: A level flag in place of two

- **WHEN** a caller needs a nonzero exit for warnings
- **THEN** hakkiri takes `--fail-level warning`, and Vale takes `--minAlertLevel` to hide alerts and `--no-exit` to keep the exit 0 ([check 8][check-8])

#### Scenario: Flags hakkiri does not have

- **WHEN** a caller gives `--glob`, `--filter`, `--ext`, `--sort`, `--relative` or `--no-global`
- **THEN** hakkiri refuses the flag as unknown and exits 2, and Vale reads each ([check 1][check-1])

### Requirement: Globs match from the project root

hakkiri SHALL match each `files` glob against the file's path relative to the project root, where Vale 3.26.0 matches the path as given after cleaning it.

#### Scenario: A file named by its absolute path

- **WHEN** `files` holds `'docs/**'` and a caller names `docs/a.md` by its absolute path
- **THEN** hakkiri lints it with the styles of that glob, and Vale matches only a glob such as `**/docs/**` ([check 3][check-3])

#### Scenario: A run from a subdirectory

- **WHEN** a caller runs from `docs/` with the config named by `HAKKIRI_CONFIG`, and checks `a.md`
- **THEN** hakkiri matches `docs/a.md`, and Vale matches `a.md`, so `[docs/**]` does not apply ([check 3][check-3])

## MODIFIED Requirements

### Requirement: Known gaps are listed apart

This spec SHALL list each thing Vale 3.26.0 does that hakkiri does not yet do, apart from the intended differences, with its queue row.  
A pull request that closes a gap SHALL remove its scenario, and record any difference the closing leaves as an intended difference.

#### Scenario: Text inside an HTML block

- **WHEN** a Markdown file holds prose inside an HTML block
- **THEN** hakkiri reads none of it, and Vale lints it with the scope of its block
- **AND** Vale also lints the text after a closed raw-text tag or a `--!>`
- **AND** Vale's entity unescaping, masking of `code`, `tt`, `kbd` and `samp`, and handling of `pre` are not yet probed (row [HKR-021][ledger], [case 9][case-9])

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

[ledger]: ../../../../../tasks.yaml
[check-1]: ../../design.md#1-one-binary-on-ctl-cores-app
[check-3]: ../../design.md#3-globs-and-paths-are-relative-to-the-project-root
[check-5]: ../../design.md#5-a-language-is-read-by-extension
[check-7]: ../../design.md#7-output-is-a-line-per-alert-or-json
[check-8]: ../../design.md#8-exit-status-follows-vales-with-a-level-flag
[case-1]: ../../../archive/2026-11-10-case-and-sequence-checks/design.md#1-case-conversion-ports-proses-converters
[case-9]: ../../../archive/2026-11-10-case-and-sequence-checks/design.md#9-a-block-of-text-has-the-scope-of-one-element
[spelling-goals]: ../../../archive/2026-11-10-spelling-and-affix-checks/design.md#goals--non-goals
[tree-1]: ../../../archive/2026-11-10-document-tree/design.md#1-markdown-rs-parses-markdown-and-mdx
