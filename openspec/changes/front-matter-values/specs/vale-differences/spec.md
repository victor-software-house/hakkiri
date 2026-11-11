## ADDED Requirements

### Requirement: Front matter alerts lie at the text

hakkiri SHALL read the front matter values as Vale does, and put each alert at the text of the value, apart from the scenarios below.

#### Scenario: A folded scalar

- **WHEN** a folded block scalar holds `Folded very` and `text here` and a rule flags `here`
- **THEN** hakkiri puts the alert on the second line, and Vale puts it on the first ([front-matter-values 5][front-matter-values-5])

#### Scenario: A TOML value

- **WHEN** a TOML value reads `"A very toml title"` and a rule flags `title`
- **THEN** hakkiri puts the alert in the value, and Vale puts it on the key, since it finds the value by searching the block ([front-matter-values 5][front-matter-values-5])

#### Scenario: An explicit tag

- **WHEN** a YAML value reads `!!str tagged` and a rule flags `tagged`
- **THEN** hakkiri puts the alert at the word, and Vale puts it at the tag ([front-matter-values 5][front-matter-values-5])

[front-matter-values-5]: ../../design.md#5-where-vale-puts-an-alert
