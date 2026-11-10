## REMOVED Requirements

### Requirement: Known gaps are listed apart

**Reason**: OpenSpec cannot drop one scenario of a modified requirement, and the gap for HTML-block text closes, so the requirement is removed and added again below without that scenario.

**Migration**: Read the requirement of the same body under the ADDED section, named "Known gaps are listed apart from the differences". Its three remaining scenarios are unchanged.

## ADDED Requirements

### Requirement: Known gaps are listed apart from the differences

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

### Requirement: HTML-block text differs from Vale in detail

hakkiri SHALL read the text of an HTML block as each scenario below says, where Vale 3.26.0 reads it otherwise.

#### Scenario: An entity reads as written

- **WHEN** an HTML block, or a Markdown paragraph, holds `marker &amp; marker` and a rule matches `marker & marker`
- **THEN** hakkiri reads the bytes of the file and gives no alert, and Vale unescapes the entity and alerts ([html 4][html-4])

#### Scenario: A tag between words adds no space

- **WHEN** an HTML block holds `a<br>marker<br/>b`
- **THEN** hakkiri reads `amarkerb` and finds no word, and Vale pads the text after an inline tag with a space and reads `a marker b` ([html 4][html-4])

#### Scenario: Alt text is not text

- **WHEN** an HTML block holds `<img alt="marker alt">` and a rule has `scope: text`
- **THEN** hakkiri selects the alt text only for `scope: alt`, and Vale selects it for `text` as well ([html 4][html-4])

#### Scenario: A keyboard key is left out

- **WHEN** an HTML block holds `a <kbd>Ctrl</kbd> b` or `a <samp>x</samp> b`
- **THEN** hakkiri leaves the text out, and Vale masks it with asterisks and counts it as a word ([html 3][html-3])

#### Scenario: A caption, a description and data are not read

- **WHEN** an HTML block holds a `caption`, a `figcaption`, a `data` element, or a `meta` element with a description
- **THEN** hakkiri reads none of the text, and Vale lints it with the scope `table.caption`, `figure.caption`, `meta` or `attr.description` ([html 5][html-5])

[html-3]: ../../../html-block-text/design.md#3-what-is-left-out-follows-vales-lists
[html-4]: ../../../html-block-text/design.md#4-entities-and-inline-tags-read-as-markdown-text-does
[html-5]: ../../../html-block-text/design.md#5-a-caption-a-description-and-a-data-element-are-not-read
[ledger]: ../../../../../tasks.yaml
[case-1]: ../../../archive/2026-11-10-case-and-sequence-checks/design.md#1-case-conversion-ports-proses-converters
[spelling-goals]: ../../../archive/2026-11-10-spelling-and-affix-checks/design.md#goals--non-goals
[tree-1]: ../../../archive/2026-11-10-document-tree/design.md#1-markdown-rs-parses-markdown-and-mdx
