## ADDED Requirements

### Requirement: A link that names its address is a gap in the prose around it

The text of a Markdown link that is one run of text equal to the address of the link SHALL stay in its `link` node.  
It SHALL be a gap in the nodes around that node, since Vale reads such a link as a URL and counts no words for it.  
The address SHALL be compared as goldmark writes it into the `href`: a byte outside its safe set percent-encoded, and a `%` before two hex digits kept.  
The address of a link by reference SHALL be the destination of the first definition of its label.

#### Scenario: A link that repeats its address

- **WHEN** a Markdown file holds ``See [a.md](a.md) now.``
- **THEN** the `paragraph` node has the text `See  now.`, and the `link` node has the text `a.md`

#### Scenario: A link with another address

- **WHEN** a Markdown file holds ``See [a.md](b.md) now.``
- **THEN** the `paragraph` node has the text `See a.md now.`

#### Scenario: A link by reference

- **WHEN** a Markdown file holds ``See [a.md][a] now.`` and the definition `[a]: a.md`
- **THEN** the `paragraph` node has the text `See  now.`

#### Scenario: A link whose text needs escaping in its address

- **WHEN** a Markdown file holds ``See [ä.md](ä.md) now.`` or ``See [a b](<a b>) now.``
- **THEN** the `paragraph` node keeps the text of the link, since goldmark writes the address as `%C3%A4.md` or `a%20b` and the text differs from it

#### Scenario: A link whose text is its escaped address

- **WHEN** a Markdown file holds ``See [a%20b](a%20b) now.``
- **THEN** the `paragraph` node has the text `See  now.`
