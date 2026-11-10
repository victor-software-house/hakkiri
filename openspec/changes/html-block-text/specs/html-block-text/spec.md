## ADDED Requirements

### Requirement: The text of an HTML block is prose

hakkiri SHALL read the text inside an HTML block of a Markdown file as prose.  
A block of text SHALL end at three places.  
They are a start tag that is not inline or void, an end tag that is not inline, and the end of the HTML block.

#### Scenario: Text in a div

- **WHEN** a Markdown file holds `<div>`, a line `Text with marker.` and `</div>`
- **THEN** the tree holds a `paragraph` node for `Text with marker.`, and an `existence` rule for `marker` with `scope: paragraph` reports it

#### Scenario: Text over two lines

- **WHEN** an HTML block holds `<p>line one` and `line two</p>`
- **THEN** one `paragraph` node reads `line one` and `line two` apart by a line feed

#### Scenario: Text between inline tags

- **WHEN** an HTML block holds `<p>one <b>two</b> three</p>`
- **THEN** the paragraph reads `one two three`, and a `strong` node reads `two`

### Requirement: The scope is the outermost element

hakkiri SHALL give a block of text the scope of the outermost `li`, `blockquote`, heading, `th` or `td` among the tags open around it, and `paragraph` when there is none.

#### Scenario: A list in HTML

- **WHEN** an HTML block holds `<ul><li>marker one</li></ul>`
- **THEN** a rule with `scope: list` reports `marker`, and a rule with `scope: paragraph` does not

#### Scenario: A heading and cells

- **WHEN** an HTML block holds `<h2>marker</h2>`, `<th>marker</th>` and `<td>marker</td>`
- **THEN** the scopes are `heading.h2`, `table.header` and `table.cell`

#### Scenario: An HTML block in a list item or a quote

- **WHEN** a list item is `- <div>marker</div>` and a quote is `> <p>marker</p>`
- **THEN** the text of the item has the scope `list` and the text of the quote has `blockquote`

#### Scenario: A second block in an item

- **WHEN** a list item holds `<div>marker one</div>`, a blank line and `<div>marker two</div>`
- **THEN** the first text has the scope `list` and the second has `paragraph`

### Requirement: Some text is left out

hakkiri SHALL leave out the text of `script`, `style`, `noscript`, `iframe`, `pre`, `kbd` and `samp`, and of an element whose class is `code`, `pre` or `problematic`.  
Text that sits in an element of another class SHALL be read.

#### Scenario: A pre block

- **WHEN** an HTML block holds `<pre>marker</pre>`
- **THEN** no rule reports `marker`

#### Scenario: Keyboard text

- **WHEN** an HTML block holds `<div>a <kbd>marker</kbd> b</div>`
- **THEN** the paragraph reads `a b`

#### Scenario: A skipped class

- **WHEN** an HTML block holds `<div class="code">marker</div>` and `<div class="highlight">marker</div>`
- **THEN** only the second is reported

### Requirement: Inline elements have their scopes

hakkiri SHALL give a node of its own to the text of `a`, `strong`, `b`, `em`, `i`, `code` and `tt`, and to the `alt` of an `img`.  
The text of `code` and `tt` SHALL read as a code span in its block.

#### Scenario: A code element

- **WHEN** an HTML block holds `<p>a <code>marker code</code> b</p>`
- **THEN** a `code` node reads `marker code`, and a rule with `scope: paragraph` does not report it

#### Scenario: A link and an image

- **WHEN** an HTML block holds `<p><a href="u">marker</a> <img alt="marker alt" src="x"></p>`
- **THEN** a `link` node reads `marker`, and an `alt` node reads `marker alt`

### Requirement: Raw text ends where the tokenizer ends it

hakkiri SHALL read the text after the end tag of a raw-text element on the same line, and after `--!>` in a comment.

#### Scenario: Text after a style element

- **WHEN** an HTML block holds `<style></style > marker same line`
- **THEN** a rule with `scope: paragraph` reports `marker`

#### Scenario: Text after a bang comment

- **WHEN** an HTML block holds `<!-- c --!> marker`
- **THEN** a rule with `scope: paragraph` reports `marker`

### Requirement: Some elements are not read

hakkiri SHALL NOT read the text of `caption`, `figcaption` and `data`, a heading of a depth outside 1 to 6, or the `content` of a `meta` element.

#### Scenario: A table caption

- **WHEN** an HTML block holds `<table><caption>marker</caption></table>`
- **THEN** no rule reports `marker`

### Requirement: A block that does not map back is not read

hakkiri SHALL read no text of an HTML block when a line of it does not end as the line of the file does.

#### Scenario: Lines in a quote

- **WHEN** a quote holds `> <div>` and `> marker` and `> </div>`
- **THEN** the text reads `marker` at its place in the file, with no `>` before it
