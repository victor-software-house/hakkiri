# html-block-text Specification

## Purpose

How hakkiri reads the text of an HTML block of a Markdown file, and which of it a rule sees, as Vale does.

## Requirements

### Requirement: The text of an HTML block is prose

hakkiri SHALL read the text inside an HTML block of a Markdown file as prose.  
A block of text SHALL end at three places.  
They are a start tag that is not inline or void, an end tag that is not inline, and the end of the HTML block.  
A block element that opens inside an inline element SHALL read as inline.

#### Scenario: Text in a div

- **WHEN** a Markdown file holds `<div>`, a line `Text with marker.` and `</div>`
- **THEN** the tree holds a `paragraph` node for `Text with marker.`, and an `existence` rule for `marker` with `scope: paragraph` reports it

#### Scenario: Text over two lines

- **WHEN** an HTML block holds `<p>line one` and `line two</p>`
- **THEN** one `paragraph` node reads `line one` and `line two` apart by a line feed

#### Scenario: A block element in an inline one

- **WHEN** an HTML block holds `<p>a <span><div>marker x</div></span> b</p>`
- **THEN** one paragraph reads `a marker x b`, as the `div` opened inside the `span`

### Requirement: The scope is the outermost element

hakkiri SHALL give a block of text the scope of the outermost `li`, `blockquote`, heading, `th`, `td`, `caption` or `figcaption` among the tags open around it, `meta` when a `data` element is open, and `paragraph` when there is none.

#### Scenario: A list in HTML

- **WHEN** an HTML block holds `<ul><li>marker one</li></ul>`
- **THEN** a rule with `scope: list` reports `marker`, and a rule with `scope: paragraph` does not

#### Scenario: A heading, cells and captions

- **WHEN** an HTML block holds `<h2>marker</h2>`, `<th>marker</th>`, `<td>marker</td>`, `<caption>marker</caption>` and `<figcaption>marker</figcaption>`
- **THEN** the scopes are `heading.h2`, `table.header`, `table.cell`, `table.caption` and `figure.caption`

#### Scenario: A data element

- **WHEN** an HTML block holds `<data value="1">`, a line `marker` and `</data>`
- **THEN** a rule with `scope: meta` reports `marker`, and a rule with `scope: text` does not

#### Scenario: An HTML block in a list item or a quote

- **WHEN** a list item is `- <div>marker</div>` and a quote is `> <p>marker</p>`
- **THEN** the text of the item has the scope `list` and the text of the quote has `blockquote`

#### Scenario: A second block in an item

- **WHEN** a list item holds `<div>marker one</div>`, a blank line and `<div>marker two</div>`
- **THEN** the first text has the scope `list` and the second has `paragraph`

### Requirement: Some text is left out or masked

hakkiri SHALL leave out the text of `script`, `style`, `noscript`, `iframe`, `pre`, and of an element whose class is `code`, `pre` or `problematic`.  
It SHALL read the first text after a `kbd` or `samp` start tag as one `*` for each character.

#### Scenario: A pre block

- **WHEN** an HTML block holds `<pre>marker</pre>`
- **THEN** no rule reports `marker`

#### Scenario: Keyboard text

- **WHEN** an HTML block holds `<div>a <kbd>Ctrl</kbd> b</div>`
- **THEN** the paragraph reads `a **** b`

#### Scenario: A skipped class

- **WHEN** an HTML block holds `<div class="code">marker</div>` and `<div class="highlight">marker</div>`
- **THEN** only the second is reported

### Requirement: Inline elements have their scopes

hakkiri SHALL give a node of its own to the text of `a`, `strong`, `b`, `em`, `i`, `code` and `tt`, and to the `alt` of an `img` and the `content` of a `meta` element that describes the page.  
The text of `code` and `tt` SHALL read as a code span in its block.  
Alt text and a description SHALL be read as `text` too, and each as one block that no sentence splitter cuts.

#### Scenario: A code element

- **WHEN** an HTML block holds `<p>a <code>marker code</code> b</p>`
- **THEN** a `code` node reads `marker code`, and a rule with `scope: paragraph` does not report it

#### Scenario: A link and an image

- **WHEN** an HTML block holds `<p><a href="u">marker</a> <img alt="marker alt" src="x"></p>`
- **THEN** a `link` node reads `marker`, and an `alt` node reads `marker alt`

#### Scenario: Alt text under text

- **WHEN** an HTML block holds `<img alt="marker alt">`, and a rule has `scope: text` or `scope: alt`
- **THEN** the rule reports `marker`

#### Scenario: A page description

- **WHEN** an HTML block holds `<meta name="description" content="marker desc">`
- **THEN** a rule with `scope: description` or `scope: text` reports `marker`

### Requirement: Tags join the text around them as Vale joins it

hakkiri SHALL join the texts of a block with a space where an inline or void tag opens before the text, and as the source has it where an inline tag or a comment closes before it.  
A text SHALL lose the whitespace at its ends, and a space SHALL not follow an opening bracket or a dash.  
Inline HTML in a Markdown paragraph SHALL join its text the same way.

#### Scenario: A tag between words

- **WHEN** an HTML block, or a Markdown paragraph, holds `a<br>marker<br/>b`
- **THEN** the text reads `a marker b`

#### Scenario: Text that touches a closed element

- **WHEN** an HTML block holds `<p>a<b>marker</b>c</p>`, `<p>a <b>marker</b> c</p>` and `<p>(<b>marker</b>)</p>`
- **THEN** the texts read `a markerc`, `a marker c` and `(marker)`

### Requirement: Raw text ends where the tokenizer ends it

hakkiri SHALL read the text after the end tag of a raw-text element on the same line, and after `--!>` in a comment.  
A `<!--` in a `script` SHALL hide its end tag until a `-->`, and a `<script` after it until the end tag of that script.

#### Scenario: Text after a style element

- **WHEN** an HTML block holds `<style></style > marker same line`
- **THEN** a rule with `scope: paragraph` reports `marker`

#### Scenario: Text after a bang comment

- **WHEN** an HTML block holds `<!-- c --!> marker`
- **THEN** a rule with `scope: paragraph` reports `marker`

#### Scenario: A script that hides its end tag

- **WHEN** an HTML block holds `<script><!-- <script> </script> marker </script> after`
- **THEN** a rule with `scope: paragraph` reports `after` and not `marker`

### Requirement: A heading of any depth Vale reads is read

hakkiri SHALL read the text of an `h0` to `h9` element under the scope `heading.h0` to `heading.h9`, as Vale reads any element named `h` and a digit.

#### Scenario: A heading of depth 7

- **WHEN** an HTML block holds `<h7>`, a line `marker` and `</h7>`
- **THEN** a rule with `scope: heading.h7`, `scope: heading` or `scope: text` reports `marker`

### Requirement: A block that does not map back is not read

hakkiri SHALL read no text of an HTML block when a line of it does not end as the line of the file does.

#### Scenario: Lines in a quote

- **WHEN** a quote holds `> <div>` and `> marker` and `> </div>`
- **THEN** the text reads `marker` at its place in the file, with no `>` before it
