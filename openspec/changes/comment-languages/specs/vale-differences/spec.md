## ADDED Requirements

### Requirement: Comments of more languages differ from Vale in these ways

hakkiri SHALL read the comments of Go, Swift, YAML, TOML and shell files, and the directives and doc links of every source language, as each scenario below says, where Vale 3.26.0 reads them otherwise.

#### Scenario: Swift is read by its own grammar

- **WHEN** a Swift file holds `/* outer /* inner */ still outer */`, a raw string `##"x "# // y"##`, a multi-line string whose line is `// z`, or `/* c */` inside a string interpolation
- **THEN** hakkiri reads `still outer` and `c` as comment text and reads nothing from the strings, and Vale reads the file with its C grammar, so it stops the first comment at the first `*/`, reads `y"##` and `z` as comments, and reads no comment inside an interpolation ([comment-languages 3][comment-languages-3])

#### Scenario: A comment that a tool reads

- **WHEN** a Swift file holds `// swiftlint:disable all`, a YAML file holds `# yaml-language-server: $schema=./a.json` or `# yamllint disable`, or a TOML file holds `#:schema ./a.json`
- **THEN** hakkiri reads none of them as prose, and Vale reads each as a sentence ([comment-languages 5][comment-languages-5])

#### Scenario: A shell file has comments

- **WHEN** a glob selects a `.sh` file
- **THEN** hakkiri reads its `#` comments, and Vale has no reader for shell, so it reads the file as plain text, or as Markdown when `formats` says so ([comment-languages 3][comment-languages-3])

#### Scenario: A YAML or TOML file needs no view

- **WHEN** a glob selects a YAML or TOML file and no view names it
- **THEN** hakkiri reads its comments, and Vale reads the whole file as plain lines, since it reads the comments of a data file only when a view names the file ([comment-languages 4][comment-languages-4])

#### Scenario: An indented line of a comment

- **WHEN** a run of line comments holds a prose line, an empty comment line, and a line indented four spaces more, such as `///     cargo run`
- **THEN** hakkiri reads the indented line as code, and Vale takes the leading spaces off every line after the first, so it reads the line as prose ([comment-languages 7][comment-languages-7])

#### Scenario: A column after a character of many bytes

- **WHEN** a Go file holds `var s = "’" // note`
- **THEN** hakkiri gives the columns of `note` in characters, and Vale adds the extra bytes of `’` to each column of a comment that trails code ([comment-languages 8][comment-languages-8])

[comment-languages-3]: ../../design.md#3-the-lexer-is-the-grammars-probed-against-vale
[comment-languages-4]: ../../design.md#4-yaml-and-toml-comments-need-no-view
[comment-languages-5]: ../../design.md#5-a-comment-for-a-tool-is-not-prose
[comment-languages-7]: ../../design.md#7-a-line-comment-keeps-the-indentation-of-its-text
[comment-languages-8]: ../../design.md#8-a-column-counts-characters
