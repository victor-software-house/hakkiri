## ADDED Requirements

### Requirement: hakkiri adds the scopes title and image

hakkiri SHALL select as each scenario below says, where Vale 3.26.0 has no such scope.

#### Scenario: A link title

- **WHEN** a rule has `scope: title` and a link holds the token in its title
- **THEN** hakkiri reports the alert, and Vale reports it only for a rule on `raw` ([link-title 1][link-title], row [HKR-054][ledger])

#### Scenario: A title selected by the element that holds it

- **WHEN** a rule has `scope: title & link` or `scope: title & image`
- **THEN** hakkiri selects only the titles of links, or only those of images, and Vale has no scope that reads a title ([link-title 5][link-title])

#### Scenario: An HTML title attribute

- **WHEN** a Markdown file holds `<p title="a marker">Some text.</p>`, a line of text with an inline tag that has a title, or an MDX file holds a JSX element with `title`, and a rule has `scope: title`
- **THEN** hakkiri selects each title, and Vale reads the attribute only in a rule on `raw` ([link-title 6][link-title])

#### Scenario: A title attribute of an a tag and an img tag

- **WHEN** an HTML block holds `<a href="u" title="a marker">x</a>` and a rule has `scope: title & link`, or `<img src="i.png" title="a marker">` and `scope: title & image`
- **THEN** hakkiri selects `a marker`, and Vale loads the rule and selects nothing, since only `raw` reads the attribute ([link-title 6][link-title])

#### Scenario: The alt text of an image

- **WHEN** a rule has `scope: image`
- **THEN** hakkiri selects the alt text of each image, and Vale loads the rule and selects nothing, since no block has an `image` part ([link-title 5][link-title])

[ledger]: ../../../../../tasks.yaml
[link-title]: ../../design.md#decisions
