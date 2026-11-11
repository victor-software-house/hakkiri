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
A code span, a comment and the `raw` text SHALL read as written.

#### Scenario: A reference in a paragraph

- **WHEN** a rule has `scope: text` and `tokens: ['marker & marker']`, and the input holds `marker &amp; marker` in a paragraph or in an HTML block
- **THEN** hakkiri reports one alert over the whole text

#### Scenario: A reference without a semicolon

- **WHEN** a paragraph holds `a &copy b`
- **THEN** the paragraph reads `a © b`

#### Scenario: A numeric reference with one digit

- **WHEN** a paragraph holds `a &#5 b`, `c &#5; d` and `e &#xA f`
- **THEN** the references read as the characters U+0005, U+0005 and a line feed, as Vale reads them, and `&#`, `&#x` and `&#xg` stay as written

#### Scenario: A reference in a code span

- **WHEN** a code span holds `&amp;` and a rule with `scope: code` matches `&amp;`
- **THEN** hakkiri reports one alert

### Requirement: A reference reads once and keeps its place

A reference SHALL read once, so `&amp;lt;` reads as `&lt;`.  
An alert on a character that a reference gave SHALL point at the first byte of the reference.

#### Scenario: A reference of a reference

- **WHEN** a paragraph holds `&amp;lt; marker`
- **THEN** the paragraph reads `&lt; marker`

#### Scenario: An alert on a decoded character

- **WHEN** a paragraph holds `a &#38; b` and a rule matches `&`
- **THEN** hakkiri reports the alert at the column of the `&` that starts `&#38;`

#### Scenario: A reference that stands for a blank at an end of a text

- **WHEN** an HTML block holds `<p>alpha&#32;</p>` and `<p>&#32;beta</p>`
- **THEN** the paragraphs read `alpha` and `beta`, as Vale trims the blank that the reference gave

#### Scenario: Alt text that is only whitespace

- **WHEN** an HTML block holds `<img src="x" alt=" ">`, or a Markdown paragraph holds `![ ](x.png)`
- **THEN** a rule with `scope: alt` reads the whitespace, as Vale does
