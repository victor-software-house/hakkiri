# vale-differences Specification

## Purpose

This spec lists every intended difference between hakkiri and Vale 3.26.0.  
Each scenario names the case, what Vale does, what hakkiri does, and links the decision that records it.  
A known gap, where hakkiri lacks something Vale does and means to close it, is not a difference.  
It is listed apart, with its queue row, under Known gaps.

## Requirements

### Requirement: Every intended difference is listed here

Every intended difference from Vale 3.26.0 SHALL be a scenario of this spec, with a link to the decision that records it.  
A pull request that adds, changes or removes a difference SHALL change this spec in the same pull request.

#### Scenario: A change adds a difference

- **WHEN** a change decides that hakkiri reports, refuses or measures something otherwise than Vale does
- **THEN** the same pull request adds a scenario for it to this spec, and links its decision

#### Scenario: A later decision removes a difference

- **WHEN** a decision makes hakkiri match Vale where an earlier decision differed, as [case 9][case-9] did for the `~` before a scope that [format 7][format-7] listed
- **THEN** the same pull request removes or rewrites the scenario, so the list holds only what is true now

### Requirement: What a rule reads differs from Vale

hakkiri SHALL read the text and the scopes of a document as each scenario below says, where Vale 3.26.0 reads them otherwise.

#### Scenario: A code span counts one word

- **WHEN** a sentence of two words, a four-word code span and one more word is measured
- **THEN** hakkiri counts 4 words and Vale counts 3 ([format 7][format-7])

#### Scenario: Rust block comments are read

- **WHEN** a Rust file holds a `/* */`, `/** */` or `/*! */` comment
- **THEN** hakkiri reads it as `text.comment.block` or `text.comment.doc.block`, and Vale reads line comments only ([format 7][format-7])

#### Scenario: String literals are never prose

- **WHEN** a code file holds a string literal
- **THEN** hakkiri never reads it as prose, and Vale reads it with the `raw` scope ([format 7][format-7])

#### Scenario: A scope part matches by dotted prefix

- **WHEN** a rule has `scope: h1` or `scope: comment.text`
- **THEN** hakkiri refuses the name as unknown, and Vale compares the set of dotted parts, so it selects the nodes that hold them ([engine 3][engine-3])

#### Scenario: A sentence in a paragraph is selected

- **WHEN** a rule has `scope: sentence & paragraph`
- **THEN** hakkiri selects the sentences of paragraphs, and Vale selects none ([case 9][case-9])

#### Scenario: A negated paragraph is left out

- **WHEN** a rule has `scope: ~paragraph`
- **THEN** hakkiri leaves paragraphs out, and Vale leaves out nothing, so hakkiri gives 104 alerts fewer on the nine files of the scope test ([case 9][case-9])

#### Scenario: An inline exclusion belongs to its scope item

- **WHEN** a rule has `scope: [heading, 'paragraph & ~link']`
- **THEN** a link in a heading still reads as text in hakkiri, and Vale blanks it, since it gathers the exclusions of every item into one list ([engine 17][engine-17])

#### Scenario: Sentences are cut by hakkiri's splitter

- **WHEN** a text has a break that the two splitters cut differently
- **THEN** hakkiri cuts it by its own splitter, which is held to split at least as well as Vale on every evaluation set, so a few inputs read apart from Vale ([splitter 5][splitter-5], [engine 12][engine-12])

### Requirement: Where an alert points and what it matches differs from Vale

hakkiri SHALL place an alert, and match text for `existence`, `substitution` and `occurrence`, as each scenario below says, where Vale 3.26.0 does otherwise.

#### Scenario: An alert is placed by offset

- **WHEN** a phrase appears twice in a block and the second copy matches
- **THEN** hakkiri points at the second copy through its byte table, and Vale's default path searches for the matched text, so it can land on the first ([engine 7][engine-7])

#### Scenario: A measurement alert points at its node

- **WHEN** a measurement alert fires on a list item
- **THEN** hakkiri points at the first byte of the measured node, past its bullet, and Vale points at column 1 of the block's first line ([engine 16][engine-16])

#### Scenario: Word and space classes are Unicode

- **WHEN** a pattern uses `\w`, `\d` or `\s` on `naïve`
- **THEN** hakkiri reads one word, since these classes are Unicode as `\b` is, and Vale reads them as ASCII, so a word with an accent reads as two ([engine 16][engine-16])

#### Scenario: capitalize needs a capital to start

- **WHEN** a swap value is capitalized for a match that starts with a digit or a symbol, such as `1st`
- **THEN** hakkiri capitalizes only a match that starts with a capital letter, and Vale capitalizes the value, so `1st` swaps to `First` ([engine 16][engine-16])

#### Scenario: A swap key with a space reads a line wrap

- **WHEN** a `substitution` key holds a space and `$1` in its value, and the matched text wraps a line
- **THEN** hakkiri expands `$1` from the key as matched, and Vale expands it from the key as written, so the alert is lost ([engine 16][engine-16])

#### Scenario: An occurrence over blank text points at a word

- **WHEN** an `occurrence` rule matches blank text only
- **THEN** hakkiri points at the first word of the node, and Vale counts the matches, points at none and gives no alert ([engine 16][engine-16])

#### Scenario: An occurrence over code is not counted

- **WHEN** an `occurrence` match overlaps a code span
- **THEN** hakkiri does not count it, and Vale counts every match, then points at the first one that is not code ([engine 16][engine-16])

#### Scenario: Overlapping phrase exceptions both hold

- **WHEN** the text is `a b c`, the exceptions are the phrases `a b` and `b c`, and a rule matches `c`
- **THEN** hakkiri holds back the match, and Vale reports `c`, since its one alternation never finds `b c` after `a b` ([engine 18][engine-18])

### Requirement: Measurements differ from Vale

hakkiri SHALL measure as each scenario below says, where Vale 3.26.0 measures otherwise.

#### Scenario: A long word is over six characters

- **WHEN** a word has six characters and seven bytes, such as `naïves`
- **THEN** hakkiri does not count it as a long word, and Vale counts every word over six bytes ([engine 16][engine-16])

#### Scenario: A whole-file metric reads code comments

- **WHEN** a whole-file metric runs on a Rust file
- **THEN** hakkiri measures the prose of the comments, and Vale prints no value, since its default Rust query gives no comment a markup format ([engine 16][engine-16])

#### Scenario: A score listed twice counts once

- **WHEN** a `readability` rule has `metrics: [SMOG, SMOG]`
- **THEN** hakkiri averages the distinct scores and reports SMOG's grade, and Vale divides by the length of the list and reports half of it ([pattern 8][pattern-8])

### Requirement: Pattern checks differ from Vale

hakkiri SHALL run the pattern checks as each scenario below says, where Vale 3.26.0 runs them otherwise.

#### Scenario: State belongs to one rule

- **WHEN** one `conditional` rule captures a term, and another rule of either kind reads the same file
- **THEN** hakkiri keeps each rule's state apart, and Vale keeps one list per file, so a captured term can satisfy another `conditional` rule or count as a `consistency` form ([pattern 1][pattern-1])

#### Scenario: A consistency alert sits at the last match

- **WHEN** a `consistency` pair matches in a node whose earlier text holds the same words
- **THEN** hakkiri puts the alert at the last match through its byte table, and Vale searches for the text, so the alert lands where it first appears ([pattern 2][pattern-2])

#### Scenario: ignorecase applies to both conditional patterns

- **WHEN** a `conditional` rule has `ignorecase: true`
- **THEN** hakkiri matches `first` and `second` without regard to case, and Vale decodes the field and reads it nowhere ([pattern 7][pattern-7])

#### Scenario: Code is no match

- **WHEN** two code spans sit side by side, or a form appears only in code
- **THEN** hakkiri counts no repeat and no form seen, and Vale masks the code, so it can report a repeat the reader does not see ([pattern 9][pattern-9])

### Requirement: Rules that Vale loads are refused

hakkiri SHALL refuse at load each rule below, which Vale 3.26.0 loads.

#### Scenario: A conditional with an empty pattern

- **WHEN** a `conditional` rule has a `first` or `second` that is empty or matches the empty string
- **THEN** hakkiri refuses it, and Vale compiles both as written ([pattern 3][pattern-3])

#### Scenario: An unknown field in a consistency rule

- **WHEN** a `consistency` rule sets a field it does not define
- **THEN** hakkiri refuses it, and Vale decodes this check loosely and accepts it ([pattern 2][pattern-2])

#### Scenario: A readability metrics list that is empty or unknown

- **WHEN** a `readability` rule has an empty `metrics` list, or a name outside the five scores
- **THEN** hakkiri refuses it, and Vale adds 0 for the unknown name and still counts it in the divisor ([pattern 6][pattern-6])

#### Scenario: A capitalization match that is empty

- **WHEN** a `capitalization` rule has an empty `match`
- **THEN** hakkiri refuses it, since every text would pass, and Vale loads it ([case 1][case-1])

#### Scenario: A capitalization style that is not AP or Chicago

- **WHEN** a `capitalization` rule has `style: chicago` or any other value than `AP` or `Chicago`
- **THEN** hakkiri refuses it with an error that names the value, and Vale reads every other value as `AP` ([case 8][case-8])

#### Scenario: A sequence with nothing to start from

- **WHEN** no token of a `sequence` rule has a `pattern`, `tag` or `upos` without `negate`
- **THEN** hakkiri refuses the rule, and Vale loads it and never matches ([case 3][case-3])

#### Scenario: A sequence token with a field or count it cannot use

- **WHEN** a `sequence` token sets a field it does not define, or a `skip` or `min` below 0, or one whose steps overflow a count
- **THEN** hakkiri refuses the rule, and Vale loads it ([case 3][case-3])

#### Scenario: A sequence model

- **WHEN** a `sequence` rule sets `model`
- **THEN** hakkiri refuses it, and Vale reads another checker's tagger model ([case 4][case-4])

#### Scenario: A spelling rule with exceptions or threshold

- **WHEN** a `spelling` rule sets `exceptions`, or a `threshold` other than 0
- **THEN** hakkiri refuses it with a message that the field has no effect, and Vale loads it and reads neither ([spelling 3][spelling-3])

#### Scenario: A dictionary outside the package

- **WHEN** a rule names a dictionary, list or affix file by a path that leaves its package, or sets only one of `aff` and `dic`
- **THEN** hakkiri refuses it, and Vale also searches an absolute path, the working directory and `DICPATH`, and answers one of `aff` and `dic` with the built-in dictionary ([spelling 2][spelling-2])

#### Scenario: A dictionary in another character set

- **WHEN** a dictionary declares a `SET` other than UTF-8 and ISO 8859-1
- **THEN** hakkiri refuses it with an error that names the set, and Vale reads every set that `golang.org/x/text` knows ([case 6][case-6])

### Requirement: Ported readers differ from Vale in detail

hakkiri SHALL differ from Vale 3.26.0 in each case below, in how it reads tags, dictionaries and spelling.

#### Scenario: A sentence is tagged whole

- **WHEN** a text is cut differently by hakkiri's splitter and by prose's segmenter
- **THEN** hakkiri tags its own sentence whole and Vale tags each part apart, so the tags can differ ([case 3][case-3])

#### Scenario: A word's shape reads Rust's Unicode properties

- **WHEN** a word holds a character such as `²`
- **THEN** hakkiri reads its shape from Rust's Unicode properties and Vale from Go's categories, so the feature, and the tag, can differ ([case 5][case-5])

#### Scenario: Compound directives are left out

- **WHEN** a dictionary uses `COMPOUNDFLAG`, `COMPOUNDBEGIN`, `COMPOUNDMIDDLE` or `COMPOUNDEND`
- **THEN** hakkiri loads it and accepts no compound made by flag, and Vale accepts a German or Dutch compound ([case 6][case-6])

#### Scenario: A strip text that is not on the word makes no form

- **WHEN** an affix rule strips `y` and adds `ies`, and the word is `cat`
- **THEN** hakkiri makes no form, as Hunspell does, and Vale makes `caties` ([case 6][case-6])

#### Scenario: A curly quote keeps its position

- **WHEN** a misspelt word holds a curly quote or `&rsquo;`
- **THEN** hakkiri reports it where it stands in the file, as written, and Vale gives it no position and reports the straight form ([spelling 1][spelling-1])

#### Scenario: An email address is not read

- **WHEN** a text holds an email address and `custom` turns the built-in filters off
- **THEN** hakkiri reads no word in it, since it is a gap in the prose around it, and Vale reports it ([spelling 1][spelling-1])

#### Scenario: Two swap keys expand to the same text

- **WHEN** an `affix` rule's `swap` has two keys whose forms expand to the same text
- **THEN** hakkiri keeps both in the order written, so the first gives the replacement, and Vale keeps the key its map visits last, which can change from run to run ([spelling 4][spelling-4])

#### Scenario: A language other than English

- **WHEN** a project checks text in a language other than English
- **THEN** hakkiri does not support it, which the spelling change states as a non-goal, and Vale reads a dictionary for the language ([spelling goals][spelling-goals])

### Requirement: A run finishes where Vale panics, stalls or runs out of memory

hakkiri SHALL finish a run on each input below in bounded time and memory, where Vale 3.26.0 panics, runs out of memory, or takes quadratic, cubic or exponential time.

#### Scenario: A pattern reaches its limit

- **WHEN** a `fancy-regex` pattern reaches its backtrack limit on a file
- **THEN** hakkiri gives one error alert for that rule and file, and Vale has no time limit and panics when a match fails ([engine 8][engine-8], [bootstrap 8][bootstrap-8])

#### Scenario: A large skip and min

- **WHEN** a `sequence` token has `skip: 1000` and `min: 1000`
- **THEN** hakkiri counts the steps and stores none, and Vale ran out of memory in 13 seconds with a 6 GB limit ([case 3][case-3])

#### Scenario: A sentence whose anchor matches most words

- **WHEN** a `sequence` sentence of 16,667 words has an anchor that matches most of them
- **THEN** hakkiri reads it in 31 ms by taking the next word the anchor accepts, and Vale costs the cube of the words, which took it 18 seconds for 8,000 ([case 3][case-3])

#### Scenario: A token of many hyphenated parts

- **WHEN** a `spelling` token holds many hyphenated parts
- **THEN** hakkiri keeps the result of each part for each depth, so it takes polynomial time, and Vale checks every part again at each split, so it takes exponential time ([case 6][case-6])

#### Scenario: A token of many marks

- **WHEN** a text holds a token of 20,000 marks
- **THEN** hakkiri splits its prefixes, suffixes and contractions in one pass, and prose searches what is left again after each one it takes off, which takes quadratic time ([case 5][case-5])

### Requirement: A comment reads as Markdown

hakkiri SHALL read every comment of a code file as Markdown, where Vale 3.26.0 reads it as plain lines unless a View names a markup format.

#### Scenario: A code span in a plain comment

- **WHEN** a Rust file holds ``// Use `utilize` here.`` and an `existence` rule matches `utilize`
- **THEN** hakkiri reads the span as code and reports nothing, and Vale reports `utilize` inside the backticks ([format 8][format-8])

### Requirement: Rule files differ from Vale in what they accept or select

hakkiri SHALL load, refuse or select as each scenario below says, where Vale 3.26.0 does otherwise.

#### Scenario: A table row is a scope

- **WHEN** a rule has `scope: table.row` and a row holds the matched text
- **THEN** hakkiri selects the text of the row, and Vale loads the rule and selects nothing, since no block has a `row` part ([format 9][format-9])

#### Scenario: A scope no node holds

- **WHEN** a rule has `scope: nonsense`
- **THEN** hakkiri refuses it as an unknown scope, and Vale loads it and selects nothing ([engine 3][engine-3])

#### Scenario: A metric counts tokens

- **WHEN** a `metric` rule sets `tokens: [";"]` and the formula `occurrences / words * 1000`
- **THEN** hakkiri measures the rate of the matches, and Vale refuses `tokens` as an invalid key and `occurrences` as an unresolved reference ([format 4][format-4], [engine 11][engine-11])

#### Scenario: A formula outside the listed syntax

- **WHEN** a `metric` rule has the formula `words > 1 ? 7.0 : 0.0`
- **THEN** hakkiri refuses it, since it reads numbers, variables, operators, `math` functions and `round`, and Vale evaluates one Tengo expression and reports 7.00 ([engine 14][engine-14])

#### Scenario: A script rule

- **WHEN** a rule has `extends: script`
- **THEN** hakkiri refuses it, and Vale runs its Tengo program ([format 5][format-5])

### Requirement: Configuration and packages are hakkiri's own

hakkiri SHALL read its settings from `.ctl/hakkiri.yaml` and take Vale styles only through `hakkiri import vale`, where Vale reads `.vale.ini`.

#### Scenario: A Vale configuration

- **WHEN** a project holds a `.vale.ini` and no `.ctl/hakkiri.yaml`
- **THEN** hakkiri reads no setting from it, and Vale reads it ([bootstrap 1][bootstrap-1])

#### Scenario: A rule is turned off

- **WHEN** a glob in the project config turns a rule off
- **THEN** it sets the rule to `off`, where Vale writes `NO` ([format 6][format-6])

#### Scenario: A package is fetched on use

- **WHEN** a project pins a package by version and hash
- **THEN** hakkiri fetches and caches it with no separate sync step, where Vale needs `vale sync` ([bootstrap 5][bootstrap-5])

### Requirement: The command line differs from Vale's

hakkiri SHALL print, report and exit as each scenario below says, where Vale 3.26.0 does otherwise.

#### Scenario: A line carries its level

- **WHEN** a Markdown file draws one warning
- **THEN** hakkiri prints `path:line:column: level Rule: message` and a count line, and Vale's `--output=line` prints `path:line:col:Rule:message` with no level ([check 7][check-7])

#### Scenario: The JSON is an object

- **WHEN** `--format json` is given
- **THEN** hakkiri prints one object holding `alerts`, `failures` and `files`, and Vale prints a map from each path to its alerts ([check 7][check-7])

#### Scenario: A level flag in place of two

- **WHEN** a caller needs a nonzero exit for warnings
- **THEN** hakkiri takes `--fail-level warning`, and Vale takes `--minAlertLevel` to hide alerts and `--no-exit` to keep the exit 0 ([check 8][check-8])

#### Scenario: Flags hakkiri does not have

- **WHEN** a caller gives `--glob`, `--filter`, `--ext`, `--sort`, `--relative` or `--no-global`
- **THEN** hakkiri refuses the flag as unknown and exits 2, and Vale reads each ([check 1][check-1])

### Requirement: Globs match from the project root

hakkiri SHALL match each `files` glob against the file's path relative to the project root, where Vale 3.26.0 matches the path as given after cleaning it.

#### Scenario: A file named by its absolute path

- **WHEN** `files` holds `'docs/**'` and a caller names `docs/a.md` by its absolute path
- **THEN** hakkiri lints it with the styles of that glob, and Vale matches only a glob such as `**/docs/**` ([check 3][check-3])

#### Scenario: A run from a subdirectory

- **WHEN** a caller runs from `docs/` with the config named by `HAKKIRI_CONFIG`, and checks `a.md`
- **THEN** hakkiri matches `docs/a.md`, and Vale matches `a.md`, so `[docs/**]` does not apply ([check 3][check-3])

### Requirement: The test command differs from Vale's

hakkiri SHALL read cases and print their report as each scenario below says, where `vale test` in Vale 3.26.0 does otherwise.

#### Scenario: A View

- **WHEN** a case sets `view`
- **THEN** hakkiri refuses the field as unknown, and Vale reads the View under its search path ([test 2][test-2])

#### Scenario: A `.yaml` file

- **WHEN** a directory holds `Rule.yaml` with `extends` and `tests`
- **THEN** hakkiri reads no case from it, and Vale reads its cases ([test 5][test-5])

#### Scenario: The report

- **WHEN** a case fails
- **THEN** hakkiri prints its file, name, reason and alerts in its own layout, and Vale prints a marked heading and a coloured diff ([test 7][test-7])

#### Scenario: A name two rule files share that no case flags

- **WHEN** `Terms.yml` and `Terms.custom.yml` both give `Team.Terms`, and `test --coverage` finds that no case flags it
- **THEN** hakkiri lists it once with both files, and Vale lists `Team.Terms` twice and names no file ([rule-names 4][rule-names-4])

### Requirement: Known gaps are listed apart from the differences

This spec SHALL list each thing Vale 3.26.0 does that hakkiri does not yet do, apart from the intended differences, with its queue row.  
A pull request that closes a gap SHALL remove its scenario, and record any difference the closing leaves as an intended difference.

#### Scenario: The built-in spelling rule

- **WHEN** a project enables Vale's `Vale.Spelling`, which ships with a default ignore list
- **THEN** hakkiri has no such rule, and a style names its own dictionary and lists (row [HKR-026][ledger], [spelling goals][spelling-goals])

#### Scenario: Vocabularies

- **WHEN** a rule sets `vocab`, or a project lists accepted and rejected terms
- **THEN** hakkiri loads the field and reads no vocabulary, and Vale adds the accepted terms to the exceptions and generates the `Vale.Terms` and `Vale.Avoid` rules (row [HKR-019][ledger], [case 1][case-1])

#### Scenario: The Markdown parser

- **WHEN** a Markdown file holds syntax that hakkiri's parser and Vale's converter read differently
- **THEN** no such difference is measured yet, so the cases are unknown (row [HKR-027][ledger], [tree 1][tree-1])

#### Scenario: A plain-text file

- **WHEN** a glob matches a file that no language reads, such as `notes.txt`
- **THEN** hakkiri skips it in a walk and refuses it when named, and Vale lints it as plain text (row [HKR-030][ledger], [check 5][check-5])

#### Scenario: Comments that turn rules off and on

- **WHEN** a file holds a `vale off` comment, a `vale on` comment or a comment that turns one rule off
- **THEN** hakkiri lints the text after it as before, and Vale skips it until the matching comment (row [HKR-031][ledger], [html 7][html-7])

#### Scenario: A rule setting that enables one rule

- **WHEN** a glob sets a rule of a style it does not list, such as `Team.Terms: warning`
- **THEN** hakkiri runs nothing for it, since a rule runs only from a listed style, and Vale runs that rule at the level the setting names (row [HKR-033][ledger])

### Requirement: HTML-block text differs from Vale in detail

hakkiri SHALL read the text of an HTML block and the text around inline HTML as each scenario below says, where Vale 3.26.0 reads it otherwise.

#### Scenario: A reference is unescaped once

- **WHEN** a paragraph holds `&amp;lt;` or `&amp;#32;`, or a code span holds `&amp;`
- **THEN** hakkiri reads `&lt;`, `&#32;` and `&amp;`, and Vale unescapes the text of each token twice, so it reads `<`, a space and `&` ([html 5][html-5])

#### Scenario: Vale loses a match beside a reference

- **WHEN** a text holds a reference that Vale cannot find in the file once it is unescaped, such as `&#109;arker` or a final `&nbsp;`, and a rule matches a word of it
- **THEN** hakkiri reports the match through its byte table, and Vale finds the words of the text in the file, so it gives no alert for a word it cannot find ([html 5][html-5])
- **AND** Vale places another alert on the wrong line or column

#### Scenario: Vale loses the text around a blank alt

- **WHEN** a paragraph holds `one marker <img src="x" alt=" "> end`
- **THEN** hakkiri reads the paragraph as `one marker end`, and Vale gives no alert for that text, though it gives one when the alt text holds a word ([html 6][html-6])

#### Scenario: A code element is a code span

- **WHEN** an HTML block holds `a <code>x</code> b`
- **THEN** hakkiri reads a code span of one word and reports no match over it, and Vale masks the text with asterisks and reports a match over them ([html 3][html-3])

#### Scenario: Inline HTML in a paragraph has no element scope

- **WHEN** a Markdown paragraph holds `a<b>marker</b>c`, `a <i>marker</i>` or an `a` element
- **THEN** hakkiri joins and masks the text around the tags as Vale does, and gives no `strong`, `emphasis` or `link` node, where Vale gives each its scope (row [HKR-027][ledger], [html 4][html-4])

#### Scenario: A block of masked text only

- **WHEN** an HTML block holds `<p><kbd>x</kbd></p>`
- **THEN** hakkiri reads `*`, and Vale reports nothing, since it cannot place text that is not in the file ([html 3][html-3])

#### Scenario: A script that never ends

- **WHEN** an HTML block holds `<script><!-- <script </script> marker`, and the next block holds `last marker`
- **THEN** hakkiri reads the next block, since it tokenizes each HTML block alone, and Vale tokenizes the whole file as one stream, so the open script hides the rest of it ([html 1][html-1])

[bootstrap-1]: ../../changes/archive/2026-11-09-bootstrap-hakkiri/design.md#1-own-format-built-from-the-best-prior-art
[bootstrap-5]: ../../changes/archive/2026-11-09-bootstrap-hakkiri/design.md#5-rules-ship-as-versioned-packages
[bootstrap-8]: ../../changes/archive/2026-11-09-bootstrap-hakkiri/design.md#8-two-regex-engines-with-a-limit
[case-1]: ../../changes/archive/2026-11-10-case-and-sequence-checks/design.md#1-case-conversion-ports-proses-converters
[case-3]: ../../changes/archive/2026-11-10-case-and-sequence-checks/design.md#3-sequence-walks-tokens-around-an-anchor
[case-4]: ../../changes/archive/2026-11-10-case-and-sequence-checks/design.md#4-model-is-refused
[case-5]: ../../changes/archive/2026-11-10-case-and-sequence-checks/design.md#5-tags-come-from-a-port-of-proses-tagger-and-model
[case-6]: ../../changes/archive/2026-11-10-case-and-sequence-checks/design.md#6-spelling-ports-vales-hunspell-reader
[case-8]: ../../changes/archive/2026-11-10-case-and-sequence-checks/design.md#8-a-style-other-than-ap-or-chicago-is-refused
[case-9]: ../../changes/archive/2026-11-10-case-and-sequence-checks/design.md#9-a-block-of-text-has-the-scope-of-one-element
[check-1]: ../../changes/archive/2026-11-10-check-command/design.md#1-one-binary-on-ctl-cores-app
[check-3]: ../../changes/archive/2026-11-10-check-command/design.md#3-globs-and-paths-are-relative-to-the-project-root
[check-5]: ../../changes/archive/2026-11-10-check-command/design.md#5-a-language-is-read-by-extension
[check-7]: ../../changes/archive/2026-11-10-check-command/design.md#7-output-is-a-line-per-alert-or-json
[check-8]: ../../changes/archive/2026-11-10-check-command/design.md#8-exit-status-follows-vales-with-a-level-flag
[engine-11]: ../../changes/archive/2026-11-10-rule-engine/design.md#11-metric-tokens-match-as-existence-tokens
[engine-12]: ../../changes/archive/2026-11-10-rule-engine/design.md#12-flesch-kincaid-parity-has-two-tolerances
[engine-14]: ../../changes/archive/2026-11-10-rule-engine/design.md#14-formulas-run-on-an-own-parser
[engine-16]: ../../changes/archive/2026-11-10-rule-engine/design.md#16-better-than-vale-where-vale-is-wrong
[engine-17]: ../../changes/archive/2026-11-10-rule-engine/design.md#17-an-inline-exclusion-belongs-to-its-scope-item
[engine-18]: ../../changes/archive/2026-11-10-rule-engine/design.md#18-every-phrase-exception-holds-overlapping-or-nested
[engine-3]: ../../changes/archive/2026-11-10-rule-engine/design.md#3-a-scope-part-matches-by-dotted-prefix
[engine-7]: ../../changes/archive/2026-11-10-rule-engine/design.md#7-positions-from-one-line-table
[engine-8]: ../../changes/archive/2026-11-10-rule-engine/design.md#8-two-regex-engines-and-a-backtrack-limit
[format-4]: ../../changes/archive/2026-11-10-rule-format/design.md#4-a-rate-is-a-metric-that-counts-tokens
[format-5]: ../../changes/archive/2026-11-10-rule-format/design.md#5-release-1-implements-every-vale-check-type-but-script
[format-6]: ../../changes/archive/2026-11-10-rule-format/design.md#6-the-project-config-uses-lowercase-keys
[format-7]: ../../changes/archive/2026-11-10-rule-format/design.md#7-intended-differences-from-vale
[format-8]: ../../changes/archive/2026-11-10-rule-format/design.md#8-every-comment-reads-as-markdown
[format-9]: ../../changes/archive/2026-11-10-rule-format/design.md#9-a-table-row-is-a-scope
[html-1]: ../../changes/archive/2026-11-10-html-block-text/design.md#1-one-walk-follows-vales
[html-3]: ../../changes/archive/2026-11-10-html-block-text/design.md#3-masked-text-reads-as-asterisks
[html-4]: ../../changes/archive/2026-11-10-html-block-text/design.md#4-tags-join-the-text-around-them
[html-5]: ../../changes/archive/2026-11-10-html-block-text/design.md#5-a-character-reference-reads-once
[html-6]: ../../changes/archive/2026-11-10-html-block-text/design.md#6-captions-data-and-descriptions-have-scopes
[html-7]: ../../changes/archive/2026-11-10-html-block-text/design.md#not-ported
[ledger]: ../../../tasks.yaml
[pattern-1]: ../../changes/archive/2026-11-10-pattern-and-readability-checks/design.md#1-per-file-state-belongs-to-one-rule
[pattern-2]: ../../changes/archive/2026-11-10-pattern-and-readability-checks/design.md#2-consistency-matches-as-existence-does-one-pattern-per-pair
[pattern-3]: ../../changes/archive/2026-11-10-pattern-and-readability-checks/design.md#3-conditional-has-two-modes-chosen-by-the-second-pattern
[pattern-6]: ../../changes/archive/2026-11-10-pattern-and-readability-checks/design.md#6-an-unknown-or-empty-metrics-list-is-refused
[pattern-7]: ../../changes/archive/2026-11-10-pattern-and-readability-checks/design.md#7-ignorecase-on-a-conditional-applies-to-both-patterns
[pattern-8]: ../../changes/archive/2026-11-10-pattern-and-readability-checks/design.md#8-a-score-listed-twice-counts-once
[pattern-9]: ../../changes/archive/2026-11-10-pattern-and-readability-checks/design.md#9-code-is-no-match-in-the-pattern-checks
[rule-names-4]: ../../changes/archive/2026-11-10-rule-names/design.md#4-test-finds-the-style-folder-as-vale-does
[spelling-1]: ../../changes/archive/2026-11-10-spelling-and-affix-checks/design.md#1-spelling-reads-words-as-vales-tokenizer-cuts-them
[spelling-2]: ../../changes/archive/2026-11-10-spelling-and-affix-checks/design.md#2-dictionaries-and-lists-stay-inside-the-package
[spelling-3]: ../../changes/archive/2026-11-10-spelling-and-affix-checks/design.md#3-fields-vale-reads-and-never-uses
[spelling-4]: ../../changes/archive/2026-11-10-spelling-and-affix-checks/design.md#4-affix-expands-before-compiling
[spelling-goals]: ../../changes/archive/2026-11-10-spelling-and-affix-checks/design.md#goals--non-goals
[splitter-5]: ../../changes/archive/2026-11-10-sentence-splitter/design.md#5-evaluation-against-gold-boundaries-not-against-vale
[test-2]: ../../changes/archive/2026-11-10-test-command/design.md#2-a-case-is-a-small-record
[test-5]: ../../changes/archive/2026-11-10-test-command/design.md#5-finding-cases
[test-7]: ../../changes/archive/2026-11-10-test-command/design.md#7-the-report-and-the-exit-status
[tree-1]: ../../changes/archive/2026-11-10-document-tree/design.md#1-markdown-rs-parses-markdown-and-mdx
