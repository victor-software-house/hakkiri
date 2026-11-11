## ADDED Requirements

### Requirement: Block and token ignores differ from Vale's in these ways

hakkiri SHALL hide the text a block or token pattern matches as Vale does, apart from the scenarios below.

#### Scenario: Sections that match one file

- **WHEN** two sections of a Vale config each list `TokenIgnores` and both match one file
- **THEN** Vale applies them in the order of a Go map, which varies from run to run, and hakkiri applies the globs in the order written ([block-token-ignores 4][block-token-ignores-4])

#### Scenario: A pattern that does not compile

- **WHEN** a glob lists a pattern that does not compile
- **THEN** hakkiri refuses the config at load with the key and its line, and Vale reports error E201 when it lints a file the section matches ([block-token-ignores 4][block-token-ignores-4])

#### Scenario: A pattern that matches no bytes

- **WHEN** a pattern is `(z*)`
- **THEN** hakkiri changes nothing, and Vale puts an empty pair of backticks at every place in the text ([block-token-ignores 2][block-token-ignores-2])

#### Scenario: Front matter

- **WHEN** a Markdown file starts with front matter and a pattern matches a `---` line
- **THEN** hakkiri leaves the front matter to the parser and the pattern sees the line, and Vale has already rewritten the front matter into a fenced block ([block-token-ignores, not ported][block-token-ignores-not-ported])

#### Scenario: A code alert after a block with no group

- **WHEN** a block pattern has no capture group, and a code span of the same text as dropped text follows the match
- **THEN** hakkiri places the alert at the code span, and Vale places it at the dropped text ([block-token-ignores, not ported][block-token-ignores-not-ported])

#### Scenario: The older block key

- **WHEN** a Vale section lists `IgnorePatterns`
- **THEN** hakkiri has no such key, and the patterns go under `block_ignores`

#### Scenario: A source file

- **WHEN** a glob that matches a Rust, TypeScript or JavaScript file lists a block or token pattern
- **THEN** hakkiri applies it to the Markdown of each comment, and Vale applies none, since it reads a comment as plain lines ([block-token-ignores 3][block-token-ignores-3])

### Requirement: Math, front matter and comment sentences are read otherwise than Vale reads them

hakkiri SHALL list here the places where it reads a file otherwise than Vale at 66bbb85, found while probing the ignores and the comment controls.

#### Scenario: Math in a Markdown file

- **WHEN** a paragraph holds `$very + really$` and a `code` rule flags `very`
- **THEN** hakkiri reads the span as text, so a text rule flags it and a code rule does not, and Vale reads it as a code span (row [HKR-048][ledger], [block-token-ignores 5][block-token-ignores-5])

#### Scenario: Front matter values

- **WHEN** a front matter block holds `title: very front` and a text rule flags `very`
- **THEN** hakkiri skips the block and reports nothing, and Vale reports `very` in the value (row [HKR-049][ledger], [block-token-ignores 5][block-token-ignores-5])

#### Scenario: A sentence rule in a comment

- **WHEN** a doc comment of a source file holds one sentence and a rule has `scope: sentence`
- **THEN** hakkiri runs the rule on the sentence, and Vale runs it on none, since it reads the comment as plain lines ([block-token-ignores 5][block-token-ignores-5])

[block-token-ignores-2]: ../../design.md#2-what-a-match-turns-into-is-what-vale-gives
[block-token-ignores-3]: ../../design.md#3-a-source-files-comments-take-the-patterns
[block-token-ignores-4]: ../../design.md#4-patterns-are-checked-when-the-config-loads
[block-token-ignores-not-ported]: ../../design.md#not-ported
[block-token-ignores-5]: ../../design.md#5-three-reads-that-differ-from-vales
[ledger]: ../../../../../tasks.yaml
