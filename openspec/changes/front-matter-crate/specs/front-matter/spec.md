## MODIFIED Requirements

### Requirement: A top-level string of front matter is text

In a Markdown or MDX file, a rule scoped `text` SHALL read each top-level string value of front matter.  
The front matter is the block at the top of the file: YAML between `---` lines, after a `---yaml` line, or closed by a `...` line; TOML between `+++` lines or after a `---toml` line; and JSON between `;;;` lines, after a `---json` line, or as an object that opens on the first line with `{`.  
The block SHALL be no part of the body: no rule reads its lines as a paragraph, a heading or a sentence.  
A key, a list, a nested mapping and a value that is not a string SHALL not be read.  
A TOML string below a table and a member of a nested JSON object SHALL not be read.  
An opening fence line MAY carry blanks after the dashes or pluses.  
A YAML key that is a list, a mapping or an alias SHALL give no value, and the entries after it SHALL be read.

#### Scenario: Values of several kinds

- **WHEN** YAML front matter holds a plain, a quoted, a folded and a literal string, and a number, a boolean, a date, a list, a nested mapping and a key that holds `very`
- **THEN** a text rule that flags `very` reports the four strings, and no other

#### Scenario: Scalars that resolve to a string

- **WHEN** the values are `yes`, `no`, `on`, `off`, `y`, `n`, `12:30`, `1,000`, `1.2.3`, `v1.0`, `1e`, `0xZZ`, `a:b` and `!!str tagged`
- **THEN** each is read as text, and `null`, `~`, `true`, `0x1f`, `1_000`, `1e3`, `.inf`, `2024-01-01`, `<<` and `!!int 3` are not

#### Scenario: Dates and the merge key

- **WHEN** the values are `2024-01-01`, `2024-01-01 10:00:00` and `<<`, plain, and then `!!str 2024-01-01` and `'2024-01-01'`
- **THEN** the plain ones are not read, and the tagged and the quoted ones are read as text

#### Scenario: Anchors, aliases and complex keys

- **WHEN** a value has an anchor, a key has an anchor, a value is an alias, and an entry has a sequence as its key
- **THEN** the anchored value and the anchored key's value are read, the alias and the entry with the sequence key are not, and the entries after them are

#### Scenario: Keys that other parsers refuse

- **WHEN** a YAML block holds a key twice, a merge key `<<: *base`, a key that is a number or a null, and the values `.inf`, `-.inf` and `.NaN`, each beside strings
- **THEN** every string is read, a key written twice gives each of its values, and no value of the block is lost

#### Scenario: Front matter that no parser reads

- **WHEN** a YAML, TOML or JSON block is not valid, such as `other: [open`
- **THEN** `hakkiri check` reports the file as a failure with the line of the problem, exits 2, reads no value of the block, and lints the rest of the file

#### Scenario: A fence with blanks

- **WHEN** an opening `---` line is followed by a space and a tab
- **THEN** the values of the block are read, with or without a block or token pattern in the glob

#### Scenario: A scalar over lines and a comment

- **WHEN** a plain scalar goes on over two lines, and another holds `very # trail`
- **THEN** the first reads as its lines joined by a space, and the second as `very`

#### Scenario: TOML and JSON

- **WHEN** TOML front matter holds `title = "A very toml title"` and a table with a string, and JSON front matter holds a string, a list and a nested object
- **THEN** the title and the top-level JSON string are read, and the table, the list and the nested object are not

#### Scenario: An MDX file

- **WHEN** an MDX file starts with YAML front matter that holds a title
- **THEN** a text rule reads the title as it does in a Markdown file

#### Scenario: Every form of block

- **WHEN** a Markdown file starts with `---yaml`, `---toml`, `---json`, a bare `{"title": "Install guide"}` line, or a YAML block closed by `...`, and the title holds `Install`
- **THEN** a text rule that flags `Install` reports it once, in the title

#### Scenario: The block is no body text

- **WHEN** a Markdown file starts with `;;;`, `{"Install": 1}` and `;;;`, and a rule scoped `paragraph` flags `Install`
- **THEN** the rule reports nothing, since the key is no value and the block is no paragraph
