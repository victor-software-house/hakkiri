## ADDED Requirements

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

### Requirement: A rule is gated by the element of a block

A rule that a comment turned off before the element of a block ends SHALL read no text in that block.  
An alert between the comment that turned a rule off and the one that turned it on SHALL be left out.  
A paragraph in a tight list item SHALL end with the item.

#### Scenario: A comment inside a paragraph

- **WHEN** a paragraph reads `C7 very really <!-- hakkiri Team.Words: off --> C8 very really good.` and no comment turns the rule on
- **THEN** `Team.Words` reports nothing in it, `C7 very` included, and nothing in the paragraphs after it

#### Scenario: A pair inside a paragraph

- **WHEN** a paragraph holds the comment that turns a rule off, then `very`, then the comment that turns it on, then `very`
- **THEN** only the second `very` draws the alert

#### Scenario: A tight list item

- **WHEN** a comment that turns a rule off is indented under the text of a tight list item, and the one that turns it on is under the item after it
- **THEN** the rule reports nothing in the first item, and nothing in the item between the comments

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
