## ADDED Requirements

### Requirement: Comments of more languages differ from Vale in these ways

hakkiri SHALL read the comments of Go, Swift, YAML, TOML and Shell files as each scenario below shows.  
It SHALL read the directives and doc links of every source language the same way, where Vale 3.26.0 reads them otherwise.

#### Scenario: Swift is read by its own grammar

- **WHEN** a Swift file contains `/* outer /* inner */ still outer */`, a raw string `##"x "# // y"##`, a multi-line string whose line is `// z`, or `/* c */` inside a string interpolation
- **THEN** hakkiri reads `still outer` and `c` as comment text and reads nothing from the strings
- **AND** Vale reads the file with its C grammar, so it stops the first comment at the first `*/`
- **AND** Vale reads `y"##` and `z` as comments, and reads no comment inside an interpolation ([comment-languages 3][comment-languages-3])

#### Scenario: A comment that a tool reads

- **WHEN** a Swift file contains `// swiftlint:disable all`, a YAML file contains `# yaml-language-server: $schema=./a.json` or `# yamllint disable`, or a TOML file contains `#:schema ./a.json`
- **THEN** hakkiri reads none of them as prose, and Vale reads each as a sentence ([comment-languages 5][comment-languages-5])

#### Scenario: A shell file has comments

- **WHEN** a glob selects a `.sh` file
- **THEN** hakkiri reads its `#` comments
- **AND** Vale has no reader for Shell, so it reads the file as plain text, or as Markdown when `formats` maps it there ([comment-languages 3][comment-languages-3])

#### Scenario: A YAML or TOML file needs no view

- **WHEN** a glob selects a YAML or TOML file and no view lists it
- **THEN** hakkiri reads its comments
- **AND** Vale reads the whole file as plain lines ([comment-languages 4][comment-languages-4])
- **AND** Vale reads the comments of a data file only when a view lists the file

#### Scenario: An indented line of a comment

- **WHEN** a run of line comments contains a prose line, an empty comment line, and a line indented four spaces more, such as `///     cargo run`
- **THEN** hakkiri reads the indented line as code
- **AND** Vale takes the leading spaces off every line after the first, so it reads the line as prose ([comment-languages 7][comment-languages-7])

#### Scenario: A see tag that opens with a link

- **WHEN** a JavaScript file contains `@see {@link Other} for more.` in a doc comment
- **THEN** hakkiri reads `for` as prose
- **AND** Vale reads the link as a type and `for` as the name of the tag, so it drops `for` ([comment-languages 6][comment-languages-6])

#### Scenario: A column after a character of many bytes

- **WHEN** a Go file contains `var s = "’" // note`
- **THEN** hakkiri gives the columns of `note` in characters
- **AND** Vale adds the extra bytes of `’` to each column of a comment that trails code ([comment-languages 8][comment-languages-8])

[comment-languages-3]: ../../design.md#3-the-lexer-is-the-grammars-probed-against-vale
[comment-languages-4]: ../../design.md#4-yaml-and-toml-comments-need-no-view
[comment-languages-5]: ../../design.md#5-a-comment-for-a-tool-is-not-prose
[comment-languages-6]: ../../design.md#6-a-doc-link-and-a-doc-name-are-masked-at-the-alert
[comment-languages-7]: ../../design.md#7-a-line-comment-keeps-the-indentation-of-the-text
[comment-languages-8]: ../../design.md#8-a-column-counts-characters
