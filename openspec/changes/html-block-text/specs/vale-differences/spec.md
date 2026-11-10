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

hakkiri SHALL read the text of an HTML block and the text around inline HTML as each scenario below says, where Vale 3.26.0 reads it otherwise.

#### Scenario: A reference is unescaped once

- **WHEN** a paragraph holds `&amp;lt;`, or a code span holds `&amp;`
- **THEN** hakkiri reads `&lt;` and `&amp;`, and Vale unescapes the text of each token twice, so it reads `<` and `&` ([html 5][html-5])

#### Scenario: Vale loses a match beside a reference

- **WHEN** a text holds a reference that Vale cannot find in the file once it is unescaped, such as `&#109;arker`, and a rule matches a word of it
- **THEN** hakkiri reports the match through its byte table, and Vale finds the words of the text in the file, so it gives no alert for a word it cannot find and places another on the wrong line or column ([html 5][html-5])

#### Scenario: A code element is a code span

- **WHEN** an HTML block holds `a <code>x</code> b`
- **THEN** hakkiri reads a code span of one word and reports no match over it, and Vale masks the text with asterisks and reports a match over them ([html 3][html-3])

#### Scenario: Inline HTML in a paragraph has no element scope

- **WHEN** a Markdown paragraph holds `a<b>marker</b>c`, `a <i>marker</i>` or an `a` element
- **THEN** hakkiri joins and masks the text around the tags as Vale does, and gives no `strong`, `emphasis` or `link` node, where Vale gives each its scope (row [HKR-027][ledger], [html 4][html-4])

#### Scenario: A heading of depth 7

- **WHEN** an HTML block holds `<h7>`, a line `marker` and `</h7>`
- **THEN** hakkiri reads no text, since it has no scope for the depth, and Vale reads a heading ([html 6][html-6])

#### Scenario: A block of masked text only

- **WHEN** an HTML block holds `<p><kbd>x</kbd></p>`
- **THEN** hakkiri reads `*`, and Vale reports nothing, since it cannot place text that is not in the file ([html 3][html-3])

#### Scenario: A script that never ends

- **WHEN** an HTML block holds `<script><!-- <script </script> marker`, and the next block holds `last marker`
- **THEN** hakkiri reads the next block, since it tokenizes each HTML block alone, and Vale tokenizes the whole file as one stream, so the open script hides the rest of it ([html 1][html-1])

[html-1]: ../../../html-block-text/design.md#1-one-walk-follows-vales
[html-3]: ../../../html-block-text/design.md#3-masked-text-reads-as-asterisks
[html-4]: ../../../html-block-text/design.md#4-tags-join-the-text-around-them
[html-5]: ../../../html-block-text/design.md#5-a-character-reference-reads-once
[html-6]: ../../../html-block-text/design.md#6-captions-data-and-descriptions-have-scopes
[ledger]: ../../../../../tasks.yaml
[case-1]: ../../../archive/2026-11-10-case-and-sequence-checks/design.md#1-case-conversion-ports-proses-converters
[spelling-goals]: ../../../archive/2026-11-10-spelling-and-affix-checks/design.md#goals--non-goals
[tree-1]: ../../../archive/2026-11-10-document-tree/design.md#1-markdown-rs-parses-markdown-and-mdx
