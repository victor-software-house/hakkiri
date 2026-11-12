# comment-controls Specification

## Purpose

An HTML comment in a Markdown file, and a `{/* … */}` comment in an MDX file, turns rules off and on for the text after it, with the reach of Vale's comments.

## Requirements

### Requirement: A comment turns every rule off and on

In a Markdown file, an HTML comment `hakkiri off` or `vale off` SHALL leave the text after it out of every block until a comment `hakkiri on` or `vale on`, or the end of the file.  
The comment SHALL be read with the blanks at its ends and its line breaks trimmed, and in capitals, with another word in front, or in a code block, a code span or after a backslash it SHALL be text.

#### Scenario: Between two blocks

- **WHEN** a paragraph holds `very really good.` between `<!-- hakkiri off -->` and `<!-- hakkiri on -->`, and so do the paragraphs before and after
- **THEN** only the paragraphs before and after draw alerts

#### Scenario: Inside a paragraph

- **WHEN** a paragraph reads `one very <!-- hakkiri off --> two very <!-- hakkiri on --> three very`
- **THEN** a rule that flags `very` reports `one` and `three`, and a rule that flags `start\s+end` reads the paragraph as `start  end` across the gap

#### Scenario: A comment that is text

- **WHEN** `<!-- Vale Off -->`, `<!-- note: vale off -->`, `\<!-- vale off -->`, a code span and a fenced block hold the words
- **THEN** no text after them is left out

#### Scenario: The blanks

- **WHEN** a comment is `<!--vale off-->`, `<!--  vale off  -->`, or spread over three lines with `vale off` on the middle one
- **THEN** it turns every rule off

#### Scenario: A comment with no partner

- **WHEN** a file holds `on` with no `off`, two `off` and one `on`, or an `off` that nothing closes
- **THEN** the first changes nothing, the second ends at the `on`, and the third runs to the end of the file, a `raw` rule included

### Requirement: A comment turns a rule or a style off and on

In a Markdown file, an HTML comment `hakkiri Style.Rule: off` or `on`, and `vale Style.Rule = NO` or `YES`, SHALL turn one rule off and on.  
The same with a style name and no rule SHALL turn the style off and on.  
Vale's `off` and `on` SHALL stand for `NO` and `YES`, and a rule name that no rule has SHALL change nothing.

#### Scenario: Between two blocks

- **WHEN** `<!-- hakkiri Team.Words: off -->` and `<!-- hakkiri Team.Words: on -->` surround a paragraph with `very really good.`
- **THEN** it draws the alerts of `Team.Really` and `Other.Good` and not that of `Team.Words`

#### Scenario: A style

- **WHEN** `<!-- hakkiri Team: off -->` surrounds a paragraph with the same words
- **THEN** it draws only the alert of `Other.Good`

#### Scenario: The spelling of a comment

- **WHEN** a comment is `vale Team.Words=NO`, `vale team.words = NO` or `vale Nope.Rule = NO`
- **THEN** it changes nothing

### Requirement: A rule is gated where its text is read

A rule that a comment turned off before the text of a block is read SHALL read no text in that block.  
The text is read when its element ends, or where a tag that opens a block inside the element flushes it.  
An alert between the comment that turned a rule off and the one that turned it on SHALL be left out.  
A paragraph in a tight list item SHALL be read before a nested list, quote, code block, table, heading or HTML block that opens a block tag, and at the end of the item otherwise.

#### Scenario: A comment inside a paragraph

- **WHEN** a paragraph reads `C7 very really <!-- hakkiri Team.Words: off --> C8 very really good.` and no comment turns the rule on
- **THEN** `Team.Words` reports nothing in it, `C7 very` included, and nothing in the paragraphs after it

#### Scenario: A pair inside a paragraph

- **WHEN** a paragraph holds the comment that turns a rule off, then `very`, then the comment that turns it on, then `very`
- **THEN** only the second `very` draws the alert

#### Scenario: A tight list item

- **WHEN** a comment that turns a rule off is indented under the text of a tight list item, and the one that turns it on is under the item after it
- **THEN** the rule reports nothing in the first item, and nothing in the item between the comments

#### Scenario: A block that opens inside a tight list item

- **WHEN** a tight list item holds `A very`, and a nested list, a quote, a fenced block, a table or a heading under it holds a comment that turns the rule off
- **THEN** `A very` draws its alert, and the text after the comment draws none

#### Scenario: A comment before the block that opens

- **WHEN** a comment that turns the rule off stands between the text of a tight list item and the nested list under it
- **THEN** the text of the item draws no alert

#### Scenario: A comment inside a nested HTML element

- **WHEN** a `div` holds `one very`, a comment that turns the rule off, and a closing `</div>`
- **THEN** `one very` draws no alert, since the closing tag reads the text after the comment

#### Scenario: A tag that opens a block inside a paragraph

- **WHEN** a paragraph reads `one very <div>two very <!-- hakkiri off --></div> three very.`
- **THEN** `one very` draws its alert, and `two very` and `three very` draw none

#### Scenario: Text left at the end of an HTML block

- **WHEN** an HTML block ends with `one very`, and a comment block that turns the rule off comes before the next paragraph
- **THEN** `one very` draws no alert, since the next block reads it after the comment

#### Scenario: A comment in a block of its own

- **WHEN** a comment that turns a rule off stands in a block quote of its own, between two paragraphs
- **THEN** the paragraph before it draws its alerts, and the paragraph after it draws none

#### Scenario: A heading

- **WHEN** a heading holds the comment that turns a rule back on, and the rule was turned off before it
- **THEN** the rule reports nothing in the heading

#### Scenario: A table cell

- **WHEN** a cell holds `very <!-- hakkiri off -->`
- **THEN** the cell draws no alert, and the cell of the next row after an `on` draws the alerts after the comment

#### Scenario: HTML blocks

- **WHEN** a `div`, a `p` or a `table` holds `very`, an `off` comment, `very`, an `on` comment and `very`
- **THEN** the first and last `very` draw alerts, and so do the words before the comment

#### Scenario: A raw rule

- **WHEN** a rule has `scope: raw`, and a file ends inside an `off`
- **THEN** it reports nothing in the file, and with a comment that turns its style off it reports nothing between the comments

### Requirement: An MDX file's braces comments are comments

In an MDX file, a comment `{/* hakkiri off */}` or `{/* vale off */}`, as a block of its own or inside a line of text, SHALL turn rules off and on as the HTML comment does, with every form of the HTML comment.  
The text SHALL be exactly `{/*` and `*/}` around the comment, so `{ /* vale off */ }` and `{/* Vale Off */}` are text.

#### Scenario: A block of its own

- **WHEN** `{/* vale off */}` and `{/* vale on */}` surround a paragraph with `very really good.`
- **THEN** it draws no alert, and the paragraphs before and after it do

#### Scenario: Inside a line of text

- **WHEN** a paragraph reads `one very {/* vale off */} two very {/* vale on */} three very`
- **THEN** a rule that flags `very` reports `one` and `three`

#### Scenario: A rule, the blanks and three lines

- **WHEN** the comments are `{/* vale Team.Words = NO */}`, `{/*vale off*/}` and `{/*` with `vale off` and `*/}` on the next two lines
- **THEN** each acts as its HTML form does

#### Scenario: A tight list item

- **WHEN** `{/* vale off */}` is indented under the text of a tight list item
- **THEN** the rules read nothing in that item, as for the HTML comment

#### Scenario: Spaces inside the braces

- **WHEN** a line reads `{ /* vale off */ }`
- **THEN** it turns nothing off
