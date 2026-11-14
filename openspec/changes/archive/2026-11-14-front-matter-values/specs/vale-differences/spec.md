## ADDED Requirements

### Requirement: Front matter alerts lie at the text

hakkiri SHALL read the front matter values as Vale does, and put each alert at the text of the value, apart from the scenarios below.

#### Scenario: A folded scalar

- **WHEN** a folded block scalar holds `Folded very` and `text here` and a rule flags `here`
- **THEN** hakkiri puts the alert on the second line, and Vale puts it on the first ([front-matter-values 5][front-matter-values-5])

#### Scenario: A TOML value

- **WHEN** a TOML value reads `"A very toml title"` and a rule flags `title`
- **THEN** hakkiri puts the alert in the value, and Vale puts it on the key, since it finds the value by searching the block ([front-matter-values 5][front-matter-values-5])

#### Scenario: An explicit tag or an anchor

- **WHEN** a YAML value reads `!!str tagged` or `&x anchored text` and a rule flags `tagged` or `anchored`
- **THEN** hakkiri puts the alert at the word, and Vale puts it at the tag or the anchor ([front-matter-values 5][front-matter-values-5])

#### Scenario: An alias

- **WHEN** a YAML value is an alias such as `*name`
- **THEN** hakkiri reads no text for it, and neither does Vale

#### Scenario: Front matter that no parser reads

- **WHEN** a block is no YAML, TOML or JSON that its parser reads, such as `other: [open`
- **THEN** hakkiri reports the file as a failure with the line of the problem, and lints the rest of the file
- **AND** Vale fails the file with its error on line 1 and lints none of it ([front-matter-values 6][front-matter-values-6])

#### Scenario: A sequence as a key

- **WHEN** a YAML block has a key that is a sequence, such as `? [a, b]`, and other entries
- **THEN** hakkiri lints the values of the other entries
- **AND** Vale fails the file with its error on line 1 and lints none of it ([front-matter-values 6][front-matter-values-6])

[front-matter-values-5]: ../../design.md#5-where-vale-puts-an-alert
[front-matter-values-6]: ../../design.md#6-a-block-no-parser-reads-is-a-fault-of-the-file
