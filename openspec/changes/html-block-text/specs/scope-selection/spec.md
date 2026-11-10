## MODIFIED Requirements

### Requirement: text selects the prose leaves

The scope `text` SHALL select each paragraph, heading, table cell and caption, in a Markdown file and inside a comment.  
It SHALL select the alt text of an image and the description of a page too, each as one block that no sentence splitter cuts.  
A list item or a block quote SHALL be read through the paragraphs inside it, so its text is read once.

#### Scenario: One alert per place

- **WHEN** a rule has `scope: text` and `tokens: [marker]`, and the input holds `# A marker`, a paragraph `A marker.` and a list item `- A marker`
- **THEN** hakkiri reports three alerts, one on each line

#### Scenario: Alt text

- **WHEN** a rule has `scope: text` and `tokens: [marker]`, and the input holds `![A marker](x.png)`
- **THEN** hakkiri reports one alert in the alt text, and a rule with `scope: sentence` reports none there

## ADDED Requirements

### Requirement: A character reference reads as its character

The text of a paragraph, heading, table cell, caption, link, strong text, emphasis and alt text SHALL read each HTML character reference as the character it stands for, as `golang.org/x/net/html` at v0.62.0 reads it.  
That covers named references, with or without the `;` for the names that HTML allows without it, and decimal and hexadecimal ones.  
A code span, a comment and the `raw` text SHALL read as written.  
A reference SHALL read once, so `&amp;lt;` reads as `&lt;`.  
An alert on a character that a reference gave SHALL point at the first byte of the reference.

#### Scenario: A reference in a paragraph

- **WHEN** a rule has `scope: text` and `tokens: ['marker & marker']`, and the input holds `marker &amp; marker` in a paragraph or in an HTML block
- **THEN** hakkiri reports one alert over the whole text

#### Scenario: A reference without a semicolon

- **WHEN** a paragraph holds `a &copy b`
- **THEN** the paragraph reads `a © b`, and an alert on `©` points at the byte of `&`

#### Scenario: A reference in a code span

- **WHEN** a code span holds `&amp;` and a rule with `scope: code` matches `&amp;`
- **THEN** hakkiri reports one alert
