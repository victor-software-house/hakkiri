## REMOVED Requirements

### Requirement: Known gaps are listed apart

**Reason**: OpenSpec cannot drop one scenario of a modified requirement, and the gap for HTML-block text closes, so the requirement is removed and added again below without that scenario.

**Migration**: Read the requirement of the same body under the ADDED section, named "Known gaps are listed apart from the differences". Its three remaining scenarios are unchanged, and it adds two.

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

#### Scenario: Comments that turn rules off and on

- **WHEN** a file holds a `vale off` comment, a `vale on` comment or a comment that turns one rule off
- **THEN** hakkiri lints the text after it as before, and Vale skips it until the matching comment (row [HKR-031][ledger], [html 7][html-7])

#### Scenario: A rule setting that enables one rule

- **WHEN** a glob sets a rule of a style it does not list, such as `Team.Terms: warning`
- **THEN** hakkiri runs nothing for it, since a rule runs only from a listed style, and Vale runs that rule at the level the setting names (row [HKR-033][ledger])

### Requirement: HTML-block text differs from Vale in detail

hakkiri SHALL read the text of an HTML block and the text around inline HTML as each scenario below says, where Vale 3.26.0 reads it otherwise.

#### Scenario: A reference is unescaped once

- **WHEN** a paragraph holds `&amp;lt;` or `&amp;#32;`, or a code span holds `&amp;`
- **THEN** hakkiri reads `&lt;`, `&#32;` and `&amp;`, and Vale unescapes the text of each token twice, so it reads `<`, a space and `&` ([html 5][html-5])

#### Scenario: Vale loses a match beside a reference

- **WHEN** a text holds a reference that Vale cannot find in the file once it is unescaped, such as `&#109;arker` or a final `&nbsp;`, and a rule matches a word of it
- **THEN** hakkiri reports the match through its byte table, and Vale finds the words of the text in the file, so it gives no alert for a word it cannot find ([html 5][html-5])
- **AND** Vale places another alert on the wrong line or column

#### Scenario: Vale loses the text around a blank alt

- **WHEN** a paragraph holds `one marker <img src="x" alt=" "> end`
- **THEN** hakkiri reads the paragraph as `one marker end`, and Vale gives no alert for that text, though it gives one when the alt text holds a word ([html 6][html-6])

#### Scenario: A code element is a code span

- **WHEN** an HTML block holds `a <code>x</code> b`
- **THEN** hakkiri reads a code span of one word and reports no match over it, and Vale masks the text with asterisks and reports a match over them ([html 3][html-3])

#### Scenario: Inline HTML in a paragraph has no element scope

- **WHEN** a Markdown paragraph holds `a<b>marker</b>c`, `a <i>marker</i>` or an `a` element
- **THEN** hakkiri joins and masks the text around the tags as Vale does, and gives no `strong`, `emphasis` or `link` node, where Vale gives each its scope (row [HKR-027][ledger], [html 4][html-4])

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
[html-7]: ../../../html-block-text/design.md#not-ported
[ledger]: ../../../../../tasks.yaml
[case-1]: ../../../archive/2026-11-10-case-and-sequence-checks/design.md#1-case-conversion-ports-proses-converters
[spelling-goals]: ../../../archive/2026-11-10-spelling-and-affix-checks/design.md#goals--non-goals
[tree-1]: ../../../archive/2026-11-10-document-tree/design.md#1-markdown-rs-parses-markdown-and-mdx
