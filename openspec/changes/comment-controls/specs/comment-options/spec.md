## ADDED Requirements

### Requirement: A comment turns off the matches of a rule or limits the styles

In a Markdown file, an HTML comment `vale Style.Rule["a","b"] = NO` or `YES` SHALL turn off and on the matches `a` and `b` of the rule.  
A comment `vale styles = A, B`, or `vale style = A`, SHALL turn off every style of the file except those it names.

#### Scenario: One match

- **WHEN** `<!-- vale Team.Words["very"] = NO -->` precedes a paragraph with `very really good.`
- **THEN** the paragraph draws `really` and `good`, and not `very`

#### Scenario: Two matches

- **WHEN** the comment lists `["really","very"]` for `Team.Words`
- **THEN** `very` is left out for `Team.Words`, and a later `YES` turns it back on

#### Scenario: Styles

- **WHEN** `<!-- vale styles = Other -->` precedes a paragraph, and `<!-- vale styles = Team, Other -->` follows
- **THEN** between them only `Other.Good` reports

### Requirement: A comment sets the level of a rule

In a Markdown file, an HTML comment `hakkiri Style.Rule: suggestion`, `warning` or `error` SHALL give each alert of the rule after it that level, over a level the config sets, until a later comment names the rule.  
A comment `hakkiri Style.Rule: on` SHALL return the level of the config.  
A level for a style, and any other word, SHALL be no comment.

#### Scenario: A level until the rule is named again

- **WHEN** a file holds `very` after no comment, after `error`, after `suggestion`, and after `on` for one rule
- **THEN** the alerts are a warning, an error, a suggestion and a warning

#### Scenario: Over the config

- **WHEN** the config sets the rule to `suggestion` and a comment before the text sets it to `error`
- **THEN** the alert is an error

### Requirement: Source files hold no comment control

A glob that matches a source file SHALL leave the text of its comments as it is, whatever HTML comment they hold.

#### Scenario: A Rust doc comment

- **WHEN** a doc comment holds `<!-- hakkiri off -->` and the lines after it hold `very`
- **THEN** the alerts are those of the file with no comment
