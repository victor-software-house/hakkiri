## ADDED Requirements

### Requirement: An inline span of math reads as code

In a Markdown or MDX file, text from a `$` to the next closing `$` SHALL read as a code span.  
The opening `$` SHALL be followed by a character that is not a space or a `$`.  
The closing `$` SHALL not be escaped, SHALL follow a character that is not a space, SHALL not be followed by a digit, and SHALL not start a line.  
A span SHALL wrap over the lines of a paragraph, and a `$` with no closing one SHALL be text.

#### Scenario: Math in prose

- **WHEN** a paragraph reads `Inline $very + really$ math with very.` and a text rule and a code rule flag `very`
- **THEN** the text rule reports the last `very` and the code rule reports the first

#### Scenario: Money

- **WHEN** a paragraph reads `Money $5 and $10 very today.`, `Glued US$5 and CA$10 very.` or `A lone $5 very here.`
- **THEN** the text rule reports `very`, since none of the three holds math

#### Scenario: Spaces and escapes

- **WHEN** a paragraph reads `Space $ very$ after.` or `Escaped \$5 very and \$10 very.`
- **THEN** the text rule reports every `very`

#### Scenario: A span over two lines

- **WHEN** a paragraph reads `Wrapped $g_i = very` and `really$ across very.` on two lines
- **THEN** the code rule reports the first `very`, and the text rule reports the second

#### Scenario: A digit after the closing delimiter

- **WHEN** a paragraph reads `Digit after $a$5 very then $b$ very.`
- **THEN** the span runs from the first `$` to the one after `b`, so the code rule reports the first `very` and the text rule the second

#### Scenario: Two dollars in a line

- **WHEN** a paragraph reads `Line $$x very$$ inline double very.`
- **THEN** the span is `$x very$`, and the code rule reports the first `very`

#### Scenario: Emphasis and a backtick inside

- **WHEN** a paragraph holds `$a *very* b$` or `$a `b` very c$`
- **THEN** the span holds them as written, and the code rule reports `very`

#### Scenario: A code span or a heading or a table cell

- **WHEN** `` `$very$` `` is a code span, and `$very$` is in a heading and in a table cell
- **THEN** the code span holds no math, and the heading and the cell each hold a span

#### Scenario: The delimiters

- **WHEN** a code rule matches `$` and a paragraph holds `$a$`
- **THEN** the rule reports both delimiters, as it does in Vale

### Requirement: A display block of math reads as code

In a Markdown or MDX file, a line of a paragraph that starts with `$$` SHALL open a block that reads as code, even in the middle of a paragraph.  
A line that ends with an unescaped `$$`, with at most one `{…}` after it, SHALL close the block, and so SHALL a blank line and the end of the paragraph.  
The text before and after the block SHALL read as separate paragraphs.

#### Scenario: A block of lines

- **WHEN** `$$`, `x = very` and `$$` are three lines between two paragraphs with `very`
- **THEN** the text rule reports only the two paragraphs

#### Scenario: One line, a label and a closer after content

- **WHEN** a block is `$$y = very$$`, ends with `$$ {#eq-a}`, or ends a content line with `$$`
- **THEN** the text rule reports nothing inside it

#### Scenario: In the middle of a paragraph

- **WHEN** `Mid para very`, a block and `tail very.` are consecutive lines
- **THEN** the text rule reports `Mid para very` and `tail very.` as two paragraphs

#### Scenario: Never closed

- **WHEN** a block opens and nothing closes it before a blank line
- **THEN** the block ends at the blank line, and the next paragraph is read

#### Scenario: In a list item and a quote

- **WHEN** a block stands under a list item or in a block quote
- **THEN** it reads as code, and the item and the quote go on around it

#### Scenario: In an MDX file

- **WHEN** an MDX file holds an inline span, money, a block and `{1 + 1}` in one line with a span
- **THEN** the alerts are those of a Markdown file, and the expression holds no math

### Requirement: A backtick beside math changes nothing

A run of backticks that opens no code span SHALL stay text beside a span, and a code span or a span that touches a span SHALL read as code with it.

#### Scenario: A tick that opens no code span

- **WHEN** a paragraph reads `` A lone ` tick $a very$ end very. `` or ``` Double ``$a very$ end very. ```
- **THEN** the code rule reports the `very` in the span, and the text rule reports the last `very`

#### Scenario: Spans that touch

- **WHEN** a paragraph reads `` Code `x`$a very$ end very. ``, `` Math $a very$`x` end very. `` or `Pair $a very$$b very$ end very.`
- **THEN** the code rule reports each `very` in a span and both `$` of each, and the text rule reports the last `very`

### Requirement: A line indented four columns opens no block

A line indented four columns or more beyond its container SHALL be text of the paragraph, and SHALL open no display block.  
A tab runs to the next multiple of four.

#### Scenario: Under no container

- **WHEN** a paragraph holds `$$`, `z very` and `$$` on lines indented four spaces or a tab, or three spaces
- **THEN** the text rule reports `z very` for four spaces and a tab, and does not for three

#### Scenario: Under a list item and a quote

- **WHEN** the lines are indented four columns past the content of a list item, or past the `>` and its space in a block quote
- **THEN** the text rule reports `w very` and `q very`, and does not for two columns

### Requirement: A paragraph of unclosed delimiters costs its length

The search for the closing `$` of a span and for the end of an HTML tag SHALL take time in proportion to the length of the paragraph.

#### Scenario: A hundred thousand unclosed openers

- **WHEN** a paragraph holds `$a` and a space, or `<a`, a hundred thousand times with no closer
- **THEN** the spans are found in seconds
