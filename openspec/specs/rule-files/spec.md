# rule-files Specification

## Purpose

A rule file holds one hakkiri rule: which text it reads, which check it runs, and the test cases that prove it.  
This spec fixes the file format, its fields, its scopes, how words are counted, and how its tests run.

## Requirements

### Requirement: A rule file holds one rule

hakkiri SHALL read each `.yml` file in a style folder as one rule, named by the folder, a dot, and the file name without its extension.

#### Scenario: The file name is the rule name

- **WHEN** a style folder `ProseStyle` holds the file `Filler.yml`
- **THEN** hakkiri loads one rule named `ProseStyle.Filler`

### Requirement: Rule files are YAML 1.2

hakkiri SHALL read a rule file as YAML 1.2, so an unquoted `yes`, `no`, `on` or `off` stays a string.

#### Scenario: A token list holds no

- **WHEN** a rule has `extends: existence`, `message: "'%s'"`, `ignorecase: true` and `tokens: [no]`, and the input is `Say no to it.`
- **THEN** hakkiri reports one alert with the message `'no'`

### Requirement: Fields keep Vale's names and meaning

Each field of a rule SHALL have the name and meaning Vale 3.26.0 gives it for the same check type, including `message`, `level`, `scope`, `tokens`, `token`, `raw`, `nonword`, `exceptions`, `ignorecase`, `swap`, `max`, `formula` and `condition`.

#### Scenario: An existence rule flags a word

- **WHEN** a rule has `extends: existence`, `message: "Filler: '%s'"`, `level: warning`, `ignorecase: true` and `tokens: [basically]`, and the input is `Basically, it works.`
- **THEN** hakkiri reports one warning with the message `Filler: 'Basically'`

#### Scenario: A substitution rule names the swap

- **WHEN** a rule has `extends: substitution`, `message: "Use '%s' instead of '%s'."` and `swap: {utilize: use}`, and the input is `We utilize it.`
- **THEN** hakkiri reports one alert with the message `Use 'use' instead of 'utilize'.`

### Requirement: Rules run every Vale check type except script

hakkiri SHALL run a rule whose `extends` is `existence`, `substitution`, `occurrence`, `metric`, `capitalization`, `consistency`, `conditional`, `repetition`, `sequence`, `spelling` or `readability`, with Vale 3.26.0's meaning for each.

#### Scenario: An occurrence rule counts a token

- **WHEN** a rule has `extends: occurrence`, `message: "%d words"`, `scope: sentence`, `max: 3` and `token: \b(\w+)\b`, and the input is `One two three four five.`
- **THEN** hakkiri reports one alert with the message `5 words`

### Requirement: A scope names text with Vale's scope names

A rule's `scope` SHALL take Vale 3.26.0's lowercase dotted scope names, such as `sentence`, `paragraph`, `heading.h1`, `list`, `blockquote`, `table.header` and `raw`.  
The rule SHALL read only the text those scopes hold.

#### Scenario: A paragraph rule skips a heading

- **WHEN** a rule has `extends: existence`, `message: "'%s'"`, `scope: paragraph` and `tokens: [marker]`, and the input is `# A marker` followed by a paragraph `Another marker.`
- **THEN** hakkiri reports one alert, on line 3, with the message `'marker'`

### Requirement: A table row is a scope

The scope `table.row` SHALL hold the text of one table row, header row included, without its pipes and padding.

#### Scenario: A long table row

- **WHEN** a rule has `extends: metric`, `message: "This table row is over 250 characters."`, `scope: table.row`, `formula: characters` and `condition: '> 250'`, and a table has one row of 260 characters of cell text and one row of 10
- **THEN** hakkiri reports one alert, on the line of the long row

### Requirement: An unknown scope is refused

hakkiri SHALL refuse a rule whose `scope` names no scope it defines, and report an error that names the rule and the scope.

#### Scenario: A scope hakkiri does not define

- **WHEN** a rule `Team.Short` has `scope: chapter`
- **THEN** hakkiri reports an error naming `Team.Short` and `chapter`, and exits with a failure status

### Requirement: Scopes combine with Vale's operators

A rule SHALL read text that every part of a `&` scope matches, and text that any item of a scope list matches.

#### Scenario: A list of scopes

- **WHEN** a rule has `scope: [heading, table.header]` and `tokens: [marker]`, and the input holds `# A marker heading`, a table whose header cell is `marker`, and a paragraph `A marker.`
- **THEN** hakkiri reports two alerts, one in the heading and one in the table header

### Requirement: An exclusion leaves out blocks of that scope

A scope part written `~` and a name SHALL leave out each block of text that has that scope, and no other.  
A block has the scope of one element, as the scope-selection spec fixes, so an element inside a quote keeps its own scope: a list item after text in a quote is not left out by `~blockquote`.

#### Scenario: A sentence in a list inside a quote

- **WHEN** a rule has `scope: sentence & ~blockquote` and `tokens: [marker]`, and the input is `A marker here.` followed by a quote that holds `> - A marker in a list.`
- **THEN** hakkiri reports one alert, on line 1, and none on the line of the list item

#### Scenario: A list item after text in a quote

- **WHEN** the same rule reads `A marker here.` followed by a quote that holds `> Quote text.`, an empty quoted line, and `> - A marker in a list.`
- **THEN** hakkiri reports two alerts, on line 1 and on the line of the list item

### Requirement: Code comments are scopes

In Rust, TypeScript and JavaScript, hakkiri SHALL read `//` comments as `text.comment.line` and `/* */` comments as `text.comment.block`; Rust's `///` and `//!` as `text.comment.doc.line`; and `/** */`, and Rust's `/*! */`, as `text.comment.doc.block`.

#### Scenario: A Rust block comment is read

- **WHEN** a rule has `scope: text.comment.block` and `tokens: [marker]`, and a Rust file holds `/* A marker here. */`
- **THEN** hakkiri reports one alert with the matched text `marker`

#### Scenario: A plain comment is not a doc comment

- **WHEN** a rule has `scope: text.comment.doc` and `tokens: [marker]`, and a Rust file holds `// A marker here.` and `/// A marker there.`
- **THEN** hakkiri reports one alert, on the `///` line

### Requirement: Every comment reads as Markdown

hakkiri SHALL read the text of every comment as Markdown, plain comments included.  
A code span, a link and a bare URL in a comment SHALL read as they do in a Markdown file.

#### Scenario: A URL in a plain comment

- **WHEN** a rule has `scope: link`, `nonword: true` and `tokens: ['^https?://\S+']`, and a Rust file holds two comment lines, `// Read https://bare.example first.` and ``// Run `https://span.example` too.``
- **THEN** hakkiri reports one alert, with the matched text `https://bare.example`

### Requirement: String literals are never prose

In a code file, every scope, `raw` included, SHALL hold comment text only, and never the text of a string literal.

#### Scenario: A word in a string

- **WHEN** a rule has `scope: raw` and `tokens: [basically]`, and a Rust file holds `let s = "basically fine";` and `// basically fine`
- **THEN** hakkiri reports one alert, on the comment line

### Requirement: A code span reads as one word

In text a rule reads, hakkiri SHALL read each code span as one word that no token matches.  
Link text SHALL read as words, and a link target SHALL be left out.

#### Scenario: A word count with a code span

- **WHEN** a rule has `extends: occurrence`, `message: "%d words"`, `scope: sentence`, `max: 1` and `token: \b(\w+)\b`, and the input is ``One two `alpha beta gamma delta` five.``
- **THEN** hakkiri reports one alert with the message `4 words`

#### Scenario: A word count with a link

- **WHEN** the same rule reads `One two [three](https://example.com/a/b/c) four five.`
- **THEN** hakkiri reports one alert with the message `5 words`

#### Scenario: A token inside a code span

- **WHEN** a rule has `tokens: [basically]` and the input is ``Run `basically` first.``
- **THEN** hakkiri reports no alert

### Requirement: A metric rule counts token matches

A `metric` rule with `tokens` SHALL give its formula the variable `occurrences`, the number of matches of those tokens in the text it measures.

#### Scenario: A semicolon rate

- **WHEN** a rule has `extends: metric`, `message: "%s semicolons per 1,000 words, over 3: split the sentences."`, `nonword: true`, `tokens: [';']`, `formula: occurrences / words * 1000` and `condition: '> 3'`, and the input is `One; two; three four five six seven eight nine ten.`
- **THEN** hakkiri reports one alert with the message `200.00 semicolons per 1,000 words, over 3: split the sentences.`

### Requirement: A metric with no scope measures the whole file

A `metric` rule with no `scope` SHALL measure the whole file once and report at most one alert for it.

#### Scenario: Two paragraphs, one rate

- **WHEN** the semicolon rule above, with no `scope`, reads two paragraphs `One; two three four five.` and `Six; seven eight nine ten.`
- **THEN** hakkiri reports one alert for the file, with the message `200.00 semicolons per 1,000 words, over 3: split the sentences.`

### Requirement: A rule carries test cases

hakkiri SHALL run each case in a rule's `tests` against that rule alone.  
It SHALL read `input` as Markdown, or as the extension that `format` names, and never assert on `about`.

#### Scenario: A case in Rust

- **WHEN** a case has `format: rs` and `input: "// A marker here."`, in a rule with `scope: text.comment` and `tokens: [marker]`
- **THEN** the case reads the input as a Rust file and the rule reports one alert

### Requirement: A test case checks the alerts

A case SHALL pass when each `contains` string appears in the rule's alerts, no `absent` string appears, and the alerts equal `want` when `want` is set.  
A `want` of `''` SHALL mean no alerts.

#### Scenario: A case that fails

- **WHEN** a case named `flags filler` has `input: "It works."` and `contains: ProseStyle.Filler`
- **THEN** hakkiri reports the case `flags filler` as failed and exits with a failure status

#### Scenario: A case that expects no alerts

- **WHEN** a case has `input: "It works."` and `want: ''`, and the rule reports no alert
- **THEN** the case passes

### Requirement: Metric tokens match as existence tokens do

The `tokens` of a `metric` rule SHALL match as the `tokens` of an `existence` rule do, with word boundaries unless `nonword` is `true`, and with `ignorecase`, `raw` and `exceptions`.

#### Scenario: A semicolon with word boundaries

- **WHEN** the semicolon rule above has no `nonword` and reads `One; two; three four five six seven eight nine ten.`
- **THEN** its formula reads `occurrences` as 0, and hakkiri reports no alert

### Requirement: A script rule is reported, and the rest of its style runs

hakkiri SHALL NOT run a rule whose `extends` is `script`.  
It SHALL report it as a rule it cannot run, with an error that names the rule file, as the project-config spec fixes.

#### Scenario: A style ships a script rule

- **WHEN** a style folder `Team` holds `Grade.yml` with `extends: script` and a glob lists `Team`
- **THEN** hakkiri reports an error naming `Team/Grade.yml` and exits 2, and the other rules of `Team` run
