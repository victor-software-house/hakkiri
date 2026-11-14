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
