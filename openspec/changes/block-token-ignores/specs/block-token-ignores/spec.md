## ADDED Requirements

### Requirement: A glob lists block and token patterns

A glob of `files` SHALL take `block_ignores` and `token_ignores`, each a list of patterns, in the project config and in a package's settings file.  
A file SHALL take the patterns of every glob that matches it, in the order the globs are written, a package's first and then the project's.  
A pattern that does not compile, or a value that is not a list, SHALL be an error that names the key and its line, and the run SHALL exit 2.

#### Scenario: A pattern that does not compile

- **WHEN** a glob holds `token_ignores: ['(']`
- **THEN** hakkiri reports an error naming `token_ignores` and the pattern, lints nothing, and exits 2

#### Scenario: A word for a list

- **WHEN** a glob holds `block_ignores: BEGIN`
- **THEN** hakkiri reports that `block_ignores` is a list of patterns, and exits 2

#### Scenario: A package and the project

- **WHEN** a package's settings file hides `very` under `'*.md'` and the project's glob `'*.md'` hides `good`
- **THEN** `a.md` holds neither alert, and with the project's glob alone it holds the alert of `very`

#### Scenario: Two globs that match one file

- **WHEN** `'text/*.md'` lists a block pattern and `'text/t2.md'` lists a token pattern
- **THEN** `text/t2.md` takes both, and the other files of `text` take the block pattern

### Requirement: A block pattern hides a match as a code block

In a Markdown or MDX file, each match of a block pattern SHALL be read as a fenced code block that holds the first group of the match, so no rule that reads prose reports text inside it.  
A match with no first group SHALL hold the text `$1` in place of the match.  
The text before and after a block in the middle of a paragraph SHALL read as two paragraphs.

#### Scenario: Two headings of an OpenSpec proposal

- **WHEN** a glob holds `block_ignores: ['(?m)^## (?:Why|What Changes)$']` and a heading rule flags `Why`, `What Changes` and `Impact`
- **THEN** only `## Impact` draws an alert

#### Scenario: A block with a group

- **WHEN** a glob holds `block_ignores: ['(?s)(BEGIN.*?END)']` and a paragraph holds `before BEGIN very mid` and `more really END after very.` on two lines
- **THEN** a rule that flags `very` and `really` reports only the `very` after `END`

#### Scenario: A block in a list item or a quote

- **WHEN** the pattern above matches inside a list item and inside a block quote
- **THEN** the text inside the match draws no alert and the text after it still does

#### Scenario: A fence inside a block

- **WHEN** `## Why` sits inside a fenced code block of the file and the block pattern of the first scenario matches it
- **THEN** the fence the rewrite adds ends the file's fence early, and hakkiri gives the alerts Vale gives for the text after it

### Requirement: A token pattern hides a match as a code span

In a Markdown or MDX file, each match of a token pattern SHALL be read as a code span that holds the first group of the match, so no rule that reads prose reports text inside it.  
A rule with `scope: code` SHALL report the text of the span.  
Block patterns SHALL run before token patterns, and each pattern SHALL run over the text the one before it left.  
A match of no bytes SHALL change nothing.

#### Scenario: Tokens in prose, a heading, a list, a quote, a table and a link

- **WHEN** a glob holds `token_ignores: ['({{.*?}})']` and each of those places holds `{{ very }}`
- **THEN** a rule that flags `very` reports none of them

#### Scenario: A code rule

- **WHEN** a rule with `scope: code` flags `very` and a token `{{ very }}` is ignored
- **THEN** it reports `very` at its place in the file, as Vale does

#### Scenario: A token that holds a block

- **WHEN** a block pattern matches the text inside `{{ BEGIN x END }}`, and a token pattern `({{.*?}})` runs after it
- **THEN** the token pattern finds no match, because the fence put a line break in it

#### Scenario: A raw rule

- **WHEN** a rule has `scope: raw` and flags `very`, and a pattern hides a line that holds it
- **THEN** the rule still reports it, since it reads the file as written

#### Scenario: A pattern that matches no bytes

- **WHEN** a block pattern and a token pattern are `(z*)`
- **THEN** the alerts are those of the file with no pattern

### Requirement: An alert points at the file as written

Every alert of a file that holds ignored text SHALL carry the line, the columns and the match it has in the file as written.

#### Scenario: A code span after a block with no group

- **WHEN** a glob holds `block_ignores: ['(?s)BEGIN.*?END']` and a code span `very` follows the match on line 7
- **THEN** a `code` rule reports it at line 7 column 12

### Requirement: Source files read no ignore

A glob that matches a source file SHALL leave its comments as they are, whatever its `block_ignores` and `token_ignores` hold.

#### Scenario: A Rust file

- **WHEN** a glob that matches `t6.rs` holds `token_ignores: ['(Why this module is unsafe)']`, and a doc comment holds that text and `very`
- **THEN** the alerts are those of the file under the same glob with no ignore
