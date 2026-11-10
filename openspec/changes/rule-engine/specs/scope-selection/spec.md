# Spec Delta

## Purpose

Selection decides which nodes of the document tree a rule reads, and what text, sentences and words a check sees in each of them.  
The [rule-files spec][rule-files] names the scopes; this spec fixes how they match.

## ADDED Requirements

### Requirement: A scope name matches by dotted prefix

A scope part SHALL select every node whose scope name equals it or starts with it followed by a dot.

#### Scenario: heading selects every level

- **WHEN** a rule has `scope: heading` and `tokens: [marker]`, and the input holds `# A marker` and `### Another marker`
- **THEN** hakkiri reports two alerts, one in each heading

#### Scenario: text.comment selects every comment

- **WHEN** a rule has `scope: text.comment` and `tokens: [marker]`, and a Rust file holds `// A marker.` and `/// Another marker.`
- **THEN** hakkiri reports two alerts

### Requirement: text selects the prose leaves

The scope `text` SHALL select each paragraph, heading and table cell, in a Markdown file and inside a comment.  
A list item or a block quote SHALL be read through the paragraphs inside it, so its text is read once.

#### Scenario: One alert per place

- **WHEN** a rule has `scope: text` and `tokens: [marker]`, and the input holds `# A marker`, a paragraph `A marker.` and a list item `- A marker`
- **THEN** hakkiri reports three alerts, one on each line

### Requirement: An unset scope reads as text

A rule of any check type except `metric` that sets no `scope` SHALL select what `text` selects.

#### Scenario: A table cell with no scope

- **WHEN** a rule has `tokens: [marker]` and no `scope`, and a table body cell holds `marker`
- **THEN** hakkiri reports one alert in that cell

### Requirement: paragraph leaves out list items and quotes

The scope `paragraph` SHALL select a paragraph that has no list item or block quote around it, as Vale 3.26.0 does.

#### Scenario: A paragraph and a list item

- **WHEN** a rule has `scope: paragraph` and `tokens: [marker]`, and the input holds `- A marker item.` and a paragraph `A marker.`
- **THEN** hakkiri reports one alert, in the paragraph

### Requirement: sentence selects each sentence

The scope `sentence` SHALL select each sentence of a paragraph, a heading or a table cell.  
A sentence SHALL keep the nodes around its node, so an exclusion such as `~blockquote` applies to it.

#### Scenario: Two sentences measured apart

- **WHEN** a rule has `extends: metric`, `message: "%s words"`, `scope: sentence`, `formula: words` and `condition: '> 3'`, and the input is `One two. Three four five six.`
- **THEN** hakkiri reports one alert, with the message `4.00 words`

### Requirement: A file extension narrows a scope

A last scope part that names a file extension, such as `rs` in `raw.rs`, SHALL limit the scope to files with that extension.

#### Scenario: raw.rs in two files

- **WHEN** a rule has `scope: raw.rs` and `tokens: [marker]`, and a Markdown file holds `A marker.` and a Rust file holds `// A marker.`
- **THEN** hakkiri reports one alert, in the Rust file

### Requirement: An inline exclusion blanks the element

A part `~link`, `~code`, `~strong` or `~emphasis` SHALL leave the selected node in place and read the text of that inline element as spaces.

#### Scenario: A word outside and inside a link

- **WHEN** a rule has `scope: text & ~link` and `tokens: [marker]`, and the input is `See marker and [marker](https://x.example).`
- **THEN** hakkiri reports one alert, at line 1, column 5

### Requirement: A check reads text without inline markup

The text a check reads SHALL hold the text of strong, emphasis and link elements without their markers, and no link target.

#### Scenario: A phrase across markup

- **WHEN** a rule has `tokens: ['bold link']`, and the input is `A **bold** [link](https://a.example) here.`
- **THEN** hakkiri reports one alert, with the matched text `bold link`, at line 1, column 5

### Requirement: A match on a code span is not reported

A match that overlaps a code span SHALL give no alert from `existence` or `substitution`.

#### Scenario: A pattern that reaches into a code span

- **WHEN** a rule has `tokens: ['run \w+']`, and the input is ``Then run `cargo test` now.``
- **THEN** hakkiri reports no alert

[rule-files]: ../../../../specs/rule-files/spec.md
