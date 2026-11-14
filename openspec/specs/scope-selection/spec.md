# scope-selection Specification

## Purpose

Selection decides which nodes of the document tree a rule reads, and what text, sentences and words a check sees in each of them.  
The [rule-files spec][rule-files] names the scopes; this spec fixes how they match.

## Requirements

### Requirement: A scope name matches by dotted prefix

A scope part SHALL select every node whose scope name equals it or starts with it followed by a dot.

#### Scenario: heading selects every level

- **WHEN** a rule has `scope: heading` and `tokens: [marker]`, and the input holds `# A marker` and `### Another marker`
- **THEN** hakkiri reports two alerts, one in each heading

#### Scenario: text.comment selects every comment

- **WHEN** a rule has `scope: text.comment` and `tokens: [marker]`, and a Rust file holds `// A marker.` and `/// Another marker.`
- **THEN** hakkiri reports two alerts

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

### Requirement: An unset scope reads as text

A rule of any check type except `metric` that sets no `scope` SHALL select what `text` selects.

#### Scenario: A table cell with no scope

- **WHEN** a rule has `tokens: [marker]` and no `scope`, and a table body cell holds `marker`
- **THEN** hakkiri reports one alert in that cell

### Requirement: paragraph leaves out list items and quotes

The scope `paragraph` SHALL select a paragraph that opens no list item or block quote, and every later paragraph of an item or a quote, as Vale 3.26.0 does.

#### Scenario: A paragraph and a list item

- **WHEN** a rule has `scope: paragraph` and `tokens: [marker]`, and the input holds `- A marker item.` and a paragraph `A marker.`
- **THEN** hakkiri reports one alert, in the paragraph

#### Scenario: A later paragraph of a list item

- **WHEN** the same rule reads the list item `- First.` followed by the paragraph `A marker here.`, indented under it
- **THEN** hakkiri reports one alert, in the later paragraph

### Requirement: sentence selects each sentence

The scope `sentence` SHALL select each sentence of a paragraph, a heading or a table cell.  
A sentence SHALL have the scope of the block it is in, so a part such as `list` selects the sentences of a list item, and an exclusion such as `~blockquote` leaves out the sentences of a quote.

#### Scenario: Two sentences measured apart

- **WHEN** a rule has `extends: metric`, `message: "%s words"`, `scope: sentence`, `formula: words` and `condition: '> 3'`, and the input is `One two. Three four five six.`
- **THEN** hakkiri reports one alert, with the message `4.00 words`

#### Scenario: A sentence in a list item

- **WHEN** a rule has `scope: sentence & list` and the input holds the paragraph `One two.` and the list item `- Three four.`
- **THEN** hakkiri selects one sentence, `Three four.`

### Requirement: An inline exclusion blanks the element

A part `~link`, `~code`, `~strong` or `~emphasis` SHALL leave the selected node in place and read the text of that inline element as spaces.
The exclusion SHALL apply only to the blocks that its own scope item selects.

#### Scenario: A word outside and inside a link

- **WHEN** a rule has `scope: text & ~link` and `tokens: [marker]`, and the input is `See marker and [marker](https://x.example).`
- **THEN** hakkiri reports one alert, at line 1, column 5

#### Scenario: An exclusion in one item

- **WHEN** a rule has `scope: [heading, 'paragraph & ~link']` and `tokens: [marker]`, and the input is a heading `# A [marker](https://x.example)` and a paragraph `See [marker](https://x.example).`
- **THEN** hakkiri reports one alert, on the heading

### Requirement: A check reads text without inline markup

The text a check reads SHALL hold the text of strong, emphasis and link elements without their markers, and no link target.

#### Scenario: A phrase across markup

- **WHEN** a rule has `tokens: ['bold link']`, and the input is `A **bold** [link](https://a.example) here.`
- **THEN** hakkiri reports one alert, with the matched text `bold link`, at line 1, column 5

### Requirement: A match on a code span is not reported

A match that overlaps a code span SHALL give no alert from `substitution`.
An `existence` rule SHALL read each code span as one `*`, a byte that no word and no space matches, and SHALL give no alert for a match that starts or ends on a code span.
A match with text on both sides of a code span SHALL give an alert, and its matched text SHALL read the span as `x`.

#### Scenario: A pattern that reaches into a code span

- **WHEN** a rule has `tokens: ['run \w+']`, and the input is ``Then run `cargo test` now.``
- **THEN** hakkiri reports no alert

#### Scenario: A match across a code span

- **WHEN** an `existence` rule has `tokens: ['Run [^.]*now']`, and the input is ``Run `cargo test` now.``
- **THEN** hakkiri reports one alert, with the matched text `Run x now`

#### Scenario: A match that starts or ends on a code span

- **WHEN** an `existence` rule has `tokens: ['[^ ]+ now']` or `tokens: ['Run [^ ]+']`, and the input is ``Run `cargo` now.``
- **THEN** hakkiri reports no alert

### Requirement: A block of text has the scope of one element

A block of text SHALL have the scope of one element, as in Vale 3.26.0.  
That element SHALL be the outermost list item, block quote, heading or table cell that the block is the first thing inside.  
A block that is the first thing inside none of them SHALL have its own scope: a paragraph, a heading or a table cell.  
A list item and a block quote SHALL be read through the blocks inside them.

#### Scenario: A heading in a quote

- **WHEN** a rule has `scope: heading` and `tokens: [marker]`, and the input is the quote `> A quote.` followed by `> # A marker heading`
- **THEN** hakkiri reports one alert, in the heading

#### Scenario: A list item in a quote

- **WHEN** a rule has `scope: list` and `tokens: [marker]`, and the input is the quote `> A quote.` followed by the list item `> - A marker item.`
- **THEN** hakkiri reports one alert, in the item

#### Scenario: A quote that opens an item

- **WHEN** a rule has `scope: list`, and the input is the list item `- > Opens the item.`
- **THEN** hakkiri selects one block, `Opens the item.`

#### Scenario: A paragraph after a fenced block in an item

- **WHEN** a rule has `scope: paragraph`, and the input is a list item that starts with a fenced block and holds the paragraph `After the fence.`
- **THEN** hakkiri selects one block, `After the fence.`

#### Scenario: A paragraph after a fenced block in a quote

- **WHEN** a rule has `scope: paragraph`, and the input is a quote that starts with a fenced block and holds the paragraph `> After the fence.`
- **THEN** hakkiri selects one block, `After the fence.`

#### Scenario: A quote inside an item

- **WHEN** a rule has `scope: paragraph`, and the input is a list item that holds a quote of the paragraphs `Para one.` and `Para two.`
- **THEN** hakkiri selects one block, `Para two.`, and a rule with `scope: list` selects `Para one.`

#### Scenario: A paragraph after a thematic break in a quote

- **WHEN** a rule has `scope: blockquote`, and the input is a quote that holds a thematic break and then the paragraph `After the rule.`
- **THEN** hakkiri selects one block, `After the rule.`

### Requirement: A dotted sentence is a sentence in a scope

A scope part written `sentence.` and a name SHALL read as that name joined to `sentence` with `&`, as in Vale 3.26.0.

#### Scenario: sentence.list

- **WHEN** a rule has `scope: sentence.list`, and the input is the paragraph `A paragraph.` and the list item `- An item.`
- **THEN** hakkiri selects one sentence, `An item.`

### Requirement: table selects cells

The scope `table` SHALL select the header and body cells of a table, and not its rows.  
The scope `table.row` SHALL select the rows.

#### Scenario: Cells and rows

- **WHEN** a rule has `scope: table`, and the input is a table of two columns with a header row and one body row
- **THEN** hakkiri selects four blocks, one for each cell

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

[rule-files]: ../rule-files/spec.md

### Requirement: A language narrows a scope

A last scope part piece that names a language by any of its langdex names, aliases or extensions, such as `rs` in `raw.rs` or `rust` in `raw.rust`, SHALL limit the scope to nodes in that language.  
A negated part with a language, such as `~heading.rs`, SHALL leave out nodes only in that language.  
A scope item whose every part names another language SHALL select nothing.

#### Scenario: raw.rs in two files

- **WHEN** a rule has `scope: raw.rs` and `tokens: [marker]`, and a Markdown file holds `A marker.` and a Rust file holds `// A marker.`
- **THEN** hakkiri reports one alert, in the Rust file

#### Scenario: One language, several extensions

- **WHEN** a rule has `scope: text.comment.js` and `tokens: [marker]`, and the files `a.js`, `a.jsx`, `a.mjs` and `a.cjs` each hold `// A marker.`
- **THEN** hakkiri reports one alert in each file

#### Scenario: A name of the language

- **WHEN** a rule has `scope: text.comment.rust` and `tokens: [marker]`, and a Rust file holds `// A marker.`
- **THEN** hakkiri reports one alert

#### Scenario: A negated part for another language

- **WHEN** a rule has `scope: text & ~heading.rs` and `tokens: [marker]`, and a Markdown file holds `# A marker`
- **THEN** hakkiri reports one alert, on the heading

#### Scenario: An item that names only another language

- **WHEN** a rule has `scope: [~heading.rs]` and `tokens: [marker]`, and a Markdown file holds `# A marker` and `A marker.`
- **THEN** hakkiri reports no alert

### Requirement: A piece names a language only after a scope

A last piece SHALL name a language only when the whole part is not a scope hakkiri defines and the rest of it is one.  
A piece that names no language SHALL be an error.

#### Scenario: A scope name that ends like a language

- **WHEN** a rule has `scope: text.comment.line`
- **THEN** hakkiri reads it as the scope of line comments, in every language

#### Scenario: A piece that names no language

- **WHEN** a rule has `scope: text.nolang`
- **THEN** hakkiri refuses the rule with `` `text.nolang` is not a scope hakkiri defines ``

### Requirement: title selects the title of a link, an image and a definition

In a Markdown or MDX file, the scope `title` SHALL select the text of the title of an inline link, an inline image and a link reference definition.
The block SHALL leave out the delimiters, and no sentence splitter SHALL cut it.
The scopes `text`, `paragraph`, `link`, `alt` and `sentence` SHALL select no part of a title.
A link or image with no title, or an empty one, SHALL give no block.

#### Scenario: Three kinds of title

- **WHEN** a rule has `scope: title` and `tokens: [marker]`, and the input holds `A [link](u.html "a marker") and ![img](i.png 'an marker') end.` and the definition `[ref]: u.html (a third marker)`
- **THEN** hakkiri reports three alerts, one in each title

#### Scenario: No prose scope reads a title

- **WHEN** a rule has one of `text`, `paragraph`, `link`, `alt` and `sentence` as its scope, and a title holds the token
- **THEN** hakkiri reports no alert in the title, as Vale does

#### Scenario: An escaped quote and an empty title

- **WHEN** the input holds `[quoted](u.html "say \"hi\" marker")` and `[empty](u.html "")`
- **THEN** `title` selects `say \"hi\" marker` and nothing for the empty title

### Requirement: A title is selected by what holds it

A scope item `title & link` SHALL select the title of an inline link, and `title & image` the title of an inline image.
The scope `link` alone and the scope `image` alone SHALL select no title, and an item with no part that names the block itself SHALL select nothing in it.
The scope `image` SHALL select the alt text of an image, and a title in a heading or a table cell SHALL be a link title as any other.

#### Scenario: Link, image and definition

- **WHEN** a link, an image and a definition each hold a title with the token `marker`
- **THEN** `title & link` reports the title of the link, `title & image` that of the image, `title` all three, and `link` and `image` none

A negated part that names the element, such as `title & ~link`, SHALL leave out the titles that element holds.

#### Scenario: A negated element

- **WHEN** a link, an image and a definition each hold a title with the token `marker`
- **THEN** `title & ~link` selects the titles of the image and the definition, and `title & ~image` those of the link and the definition

#### Scenario: A heading and a table cell

- **WHEN** a heading and a table cell each hold a link with a title
- **THEN** `title & link` selects both titles

#### Scenario: An image with no alt text

- **WHEN** an image has an empty alt text and a title
- **THEN** `title & image` selects the title, and `image` selects nothing in it

### Requirement: An HTML title attribute is a title

The `title` attribute of an HTML tag SHALL be selected by `title`, and by no prose scope.
That holds for a tag in an HTML block, a tag in a line of text, and the opening tag of a JSX element.
The attribute of an `a` tag SHALL be a link title, and that of an `img` tag an image title.

#### Scenario: An HTML block

- **WHEN** an HTML block is `<p title="an html marker">Some text.</p>`
- **THEN** `title` selects `an html marker`, and `text` selects `Some text.`

#### Scenario: A line of text and a JSX element

- **WHEN** a paragraph holds `A <span title="a line marker">x</span> here.` and an MDX file holds `<Foo title="a jsx marker" />`
- **THEN** `title` selects `a line marker` and `a jsx marker`, and `text` selects neither

#### Scenario: An a tag and an img tag

- **WHEN** an HTML block holds `<a href="u" title="an a marker">x</a>` and `<img src="i.png" title="an img marker">`
- **THEN** `title & link` selects `an a marker`, `title & image` selects `an img marker`, and `link` and `image` select neither

#### Scenario: A tag in the text of a link

- **WHEN** a link `[A <span title="a nested marker">x</span> end](u "a link marker")` holds a tag with a title in its text
- **THEN** `title` selects both, `title & link` selects `a link marker` only, and `title & ~link` selects `a nested marker`
