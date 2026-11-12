## ADDED Requirements

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

#### Scenario: A heading and a table cell

- **WHEN** a heading and a table cell each hold a link with a title
- **THEN** `title & link` selects both titles

#### Scenario: An image with no alt text

- **WHEN** an image has an empty alt text and a title
- **THEN** `title & image` selects the title, and `image` selects nothing in it

### Requirement: An HTML title attribute is a title

In an HTML block, the `title` attribute of a tag with text SHALL be selected by `title`, and by no prose scope.
In a line of text, the attribute SHALL be read by `raw` only, as in Vale.
The attribute of an `a` tag SHALL be a link title, and that of an `img` tag an image title.

#### Scenario: An HTML block

- **WHEN** an HTML block is `<p title="an html marker">Some text.</p>`
- **THEN** `title` selects `an html marker`, and `text` selects `Some text.`

#### Scenario: An a tag and an img tag

- **WHEN** an HTML block holds `<a href="u" title="an a marker">x</a>` and `<img src="i.png" title="an img marker">`
- **THEN** `title & link` selects `an a marker`, `title & image` selects `an img marker`, and `link` and `image` select neither
