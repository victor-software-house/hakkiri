## ADDED Requirements

### Requirement: title selects the title of a link, an image and a definition

In a Markdown or MDX file, the scope `title` SHALL select the text of the title of an inline link, an inline image and a link reference definition, with its delimiters left out, as one block that no sentence splitter cuts.
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
