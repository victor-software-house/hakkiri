## ADDED Requirements

### Requirement: An MDX expression and a childless JSX element read as code

In an MDX file, an `{…}` expression SHALL read as a code node over the text as written.  
This SHALL apply in a line of text, a heading, a table cell or a block of its own.

A JSX element with no children SHALL read the same way when it is a block.  
It SHALL do so too when it is written `<Tag />` in a line of text.  
A `{/* … */}` comment SHALL read as no node, and the tags of an element with children SHALL read as no node.

#### Scenario: A self-closing element in a line of text

- **WHEN** a paragraph reads `Pre <Tag a="very" /> post very.` and a code rule and a text rule flag `very`
- **THEN** the code rule reports the first `very`, and the text rule reports the last

#### Scenario: An element with children

- **WHEN** a paragraph reads `Pre <Tag a="very">child very</Tag> post very.`
- **THEN** the text rule reports `child very` and `post very`, and nothing comes from the code rule

#### Scenario: A block element with no children

- **WHEN** a block reads `<Divider label="very" />` or `<Empty label="very"></Empty>`
- **THEN** the code rule reports `very`
- **AND** an empty pair of tags inside a line of text gives no code node

#### Scenario: An expression

- **WHEN** `{"$" + very}` is a block, in a heading, in a table cell or in a line of text
- **THEN** a code rule that matches `$` and `very` reports both, and a text rule reports only the words around it

#### Scenario: A comment

- **WHEN** a block reads `{/* very $ */} then very.`
- **THEN** nothing comes from the code rule, and the text rule reports the last `very`
