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

### Requirement: A rule setting is spelled as hakkiri's config spells it

hakkiri SHALL write `on` and `off` where Vale 3.26.0 writes `YES` and `NO`, and SHALL refuse a rule key that names no rule and a value that is none of those words or a level.

#### Scenario: A rule turned on at its own level

- **WHEN** a glob turns a rule on without naming a level
- **THEN** it sets the rule to `on`, where Vale writes `YES` ([rule-setting 7][rule-setting-7])

#### Scenario: A key that names no rule

- **WHEN** a glob holds `Team.Nope: warning` and no loaded style holds a rule `Team.Nope`
- **THEN** hakkiri refuses it with the line of the key, and Vale ignores `Team.Nope = YES` ([rule-setting 7][rule-setting-7])

#### Scenario: A word that is no level

- **WHEN** a glob sets a rule to `yes`
- **THEN** hakkiri refuses it with the line, and Vale warns that `yes` is not `YES`, `NO`, `UNSET` or a level, and switches the rule off ([rule-setting 7][rule-setting-7])

### Requirement: Rule files differ from Vale in what they select

hakkiri SHALL load or select as each scenario below says, where Vale 3.26.0 does otherwise.

#### Scenario: A table row is a scope

- **WHEN** a rule has `scope: table.row` and a row holds the matched text
- **THEN** hakkiri selects the text of the row, and Vale loads the rule and selects nothing, since no block has a `row` part ([format 9][format-9])

#### Scenario: A scope no node holds

- **WHEN** a rule has `scope: nonsense`
- **THEN** hakkiri refuses it as an unknown scope, and Vale loads it and selects nothing ([engine 3][engine-3])

#### Scenario: A metric counts tokens

- **WHEN** a `metric` rule sets `tokens: [";"]` and the formula `occurrences / words * 1000`
- **THEN** hakkiri measures the rate of the matches, and Vale refuses `tokens` as an invalid key and `occurrences` as an unresolved reference ([format 4][format-4], [engine 11][engine-11])

### Requirement: Configuration is hakkiri's own, and Vale styles load as they are

hakkiri SHALL read its settings from `.ctl/hakkiri.yaml`, where Vale reads `.vale.ini`, and SHALL load a Vale style folder as it is.

#### Scenario: A Vale configuration

- **WHEN** a project holds a `.vale.ini` and no `.ctl/hakkiri.yaml`
- **THEN** hakkiri reads no setting from it, and Vale reads it ([bootstrap 1][bootstrap-1], row [HKR-035][ledger])

#### Scenario: A rule is turned off

- **WHEN** a glob in the project config turns a rule off
- **THEN** it sets the rule to `off`, where Vale writes `NO` ([format 6][format-6])

#### Scenario: A package is fetched on use

- **WHEN** a project pins a package by `url` and `sha256`
- **THEN** hakkiri fetches it on the first run, checks the hash and keeps it in a cache folder outside the project, where Vale needs `vale sync`, accepts any zip and copies the styles into `StylesPath` ([package-fetch 1][package-fetch-1])

#### Scenario: A package that was changed

- **WHEN** a pinned package's download does not match its `sha256`
- **THEN** hakkiri refuses it and names the `url` and both hashes, where Vale installs a changed zip with no complaint ([package-fetch 3][package-fetch-3])

#### Scenario: A Vale style folder

- **WHEN** a package holds a Vale style folder, such as Voices, Std or Readability
- **THEN** hakkiri loads it with no conversion, as Vale does, apart from the rules the Known gaps list ([styles 1][styles-1])

### Requirement: Vocabularies differ from Vale's in these ways

hakkiri SHALL read vocabularies as Vale does, apart from the scenarios below.

#### Scenario: A term that is not a pattern

- **WHEN** a list holds a term such as `C(` that is not a valid pattern
- **THEN** hakkiri reports an error that names the file and line, where Vale stops with a pattern error that has no line ([vocabularies 4][vocabularies-4])

#### Scenario: A term that Vale's engines refuse

- **WHEN** a list holds `C++`
- **THEN** hakkiri reads it as the pattern `C` with a possessive plus, and holds back `C` and `CC`, where Vale stops with a pattern error ([bootstrap 8][bootstrap-8])

#### Scenario: Two packages with one vocabulary

- **WHEN** two packages each hold a vocabulary `Team`
- **THEN** hakkiri refuses the config and names both, where Vale takes the first search path, or the last package after `vale sync` ([vocabularies 3][vocabularies-3])

#### Scenario: The per-user styles folder

- **WHEN** a vocabulary is in no package of the project
- **THEN** hakkiri reports it unknown, where Vale also searches its per-user styles folder ([vocabularies 3][vocabularies-3])

#### Scenario: An unknown vocabulary

- **WHEN** the config names a vocabulary that no package holds
- **THEN** hakkiri names the vocabulary and the line of the key, where Vale lists the search paths and gives no line ([vocabularies 3][vocabularies-3])

### Requirement: Package settings differ from Vale's package configuration

hakkiri SHALL read a package's settings as each scenario below says, where Vale 3.26.0 reads a package's `.vale.ini` otherwise.

#### Scenario: The file and its place

- **WHEN** a package holds `.ctl/hakkiri.yaml`
- **THEN** hakkiri reads it from the package's top folder, beside `styles`, and Vale copies the package's `.vale.ini` to `StylesPath/.vale-config/<index>-<Name>.ini` at `vale sync` ([package settings 2][settings-2])

#### Scenario: A scalar in a package

- **WHEN** a package and the project both set `level`
- **THEN** the project's value applies, and Vale lets a package win for the keys it reads first, such as `WordTemplate` and the `[formats]` entries ([package settings 5][settings-5])

#### Scenario: An unknown key in a package

- **WHEN** a package's settings file holds a key hakkiri does not define
- **THEN** hakkiri refuses it with the line and the package, and Vale warns that the key is not a core option and ignores it ([package settings 7][settings-7])

#### Scenario: A deep chain of packages

- **WHEN** the chain of packages that list packages is more than 10 deep
- **THEN** hakkiri refuses it, and Vale follows each `Packages` entry with no limit and no check for a loop ([package settings 3][settings-3])

#### Scenario: A nested package pinned by path

- **WHEN** a package's `packages` holds a `path` entry
- **THEN** hakkiri refuses it, and Vale installs a local folder named there ([package settings 3][settings-3])

#### Scenario: Ten or more packages

- **WHEN** a project lists ten or more packages
- **THEN** hakkiri applies them in the listed order, and Vale sorts their fragment files as text, so `10-` comes before `2-` ([package settings 6][settings-6])

#### Scenario: A project section that replaces package styles

- **WHEN** a project's section under another glob replaces the styles a package turned on
- **THEN** hakkiri prints nothing, and Vale warns once that the section replaces the styles of the package ([package settings 4][settings-4])

### Requirement: Vocabulary rules differ from Vale's in these ways

hakkiri SHALL build and run the rules of a vocabulary as Vale does, apart from the scenarios below.

#### Scenario: An escape in a term

- **WHEN** `data\Wset` is accepted and the text is `A data set, data-set, dataxset and DATA SET.`
- **THEN** hakkiri flags `DATA SET`, since it lowercases the literal text of the term and keeps `\W`, and Vale flags `dataxset`, since it lowercases the escape to `\w` ([vocabulary-rules 1][vocabulary-rules-1])

#### Scenario: The style Vale with no term

- **WHEN** a glob lists `styles: [Vale]` and no named vocabulary holds a term
- **THEN** hakkiri refuses the config, where Vale runs `Vale.Spelling` and `Vale.Repetition` from that style (rows [HKR-026][ledger] and [HKR-038][ledger], [vocabulary-rules 4][vocabulary-rules-4])

[bootstrap-1]: ../../changes/archive/2026-11-09-bootstrap-hakkiri/design.md#1-own-format-built-from-the-best-prior-art
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
[check-7]: ../../changes/archive/2026-11-10-check-command/design.md#7-output-is-a-line-per-alert-or-json
[check-8]: ../../changes/archive/2026-11-10-check-command/design.md#8-exit-status-follows-vales-with-a-level-flag
[engine-11]: ../../changes/archive/2026-11-10-rule-engine/design.md#11-metric-tokens-match-as-existence-tokens
[engine-12]: ../../changes/archive/2026-11-10-rule-engine/design.md#12-flesch-kincaid-parity-has-two-tolerances
[engine-14]: ../../changes/archive/2026-11-10-rule-engine/design.md#14-formulas-run-on-an-own-parser
[engine-16]: ../../changes/archive/2026-11-10-rule-engine/design.md#16-better-than-vale-where-vale-is-wrong
[engine-17]: ../../changes/archive/2026-11-10-rule-engine/design.md#17-an-inline-exclusion-belongs-to-the-scope-item
[engine-18]: ../../changes/archive/2026-11-10-rule-engine/design.md#18-every-phrase-exception-holds-overlapping-or-nested
[engine-3]: ../../changes/archive/2026-11-10-rule-engine/design.md#3-a-scope-part-matches-by-dotted-prefix
[engine-7]: ../../changes/archive/2026-11-10-rule-engine/design.md#7-positions-from-one-line-table
[engine-8]: ../../changes/archive/2026-11-10-rule-engine/design.md#8-two-regex-engines-and-a-backtrack-limit
[format-4]: ../../changes/archive/2026-11-10-rule-format/design.md#4-a-rate-is-a-metric-that-counts-tokens
[format-6]: ../../changes/archive/2026-11-10-rule-format/design.md#6-the-project-config-uses-lowercase-keys
[format-7]: ../../changes/archive/2026-11-10-rule-format/design.md#7-intended-differences-from-vale
[format-8]: ../../changes/archive/2026-11-10-rule-format/design.md#8-every-comment-reads-as-markdown
[format-9]: ../../changes/archive/2026-11-10-rule-format/design.md#9-a-table-row-is-a-scope
[html-1]: ../../changes/archive/2026-11-10-html-block-text/design.md#1-one-walk-follows-vales
[html-3]: ../../changes/archive/2026-11-10-html-block-text/design.md#3-masked-text-reads-as-asterisks
[html-4]: ../../changes/archive/2026-11-10-html-block-text/design.md#4-tags-join-the-text-around-them
[html-5]: ../../changes/archive/2026-11-10-html-block-text/design.md#5-a-character-reference-reads-once
[html-6]: ../../changes/archive/2026-11-10-html-block-text/design.md#6-captions-data-and-descriptions-have-scopes
[ledger]: ../../../tasks.yaml
[package-fetch-1]: ../../changes/archive/2026-11-11-package-fetch/design.md#1-vales-sync-is-the-reference-for-the-source
[package-fetch-3]: ../../changes/archive/2026-11-11-package-fetch/design.md#3-the-checks-a-download-passes
[pattern-1]: ../../changes/archive/2026-11-10-pattern-and-readability-checks/design.md#1-per-file-state-belongs-to-one-rule
[pattern-2]: ../../changes/archive/2026-11-10-pattern-and-readability-checks/design.md#2-consistency-matches-as-existence-does-one-pattern-per-pair
[pattern-3]: ../../changes/archive/2026-11-10-pattern-and-readability-checks/design.md#3-conditional-has-two-modes-chosen-by-the-second-pattern
[pattern-6]: ../../changes/archive/2026-11-10-pattern-and-readability-checks/design.md#6-an-unknown-or-empty-metrics-list-is-refused
[pattern-7]: ../../changes/archive/2026-11-10-pattern-and-readability-checks/design.md#7-ignorecase-on-a-conditional-applies-to-both-patterns
[pattern-8]: ../../changes/archive/2026-11-10-pattern-and-readability-checks/design.md#8-a-score-listed-twice-counts-once
[pattern-9]: ../../changes/archive/2026-11-10-pattern-and-readability-checks/design.md#9-code-is-no-match-in-the-pattern-checks
[rule-names-4]: ../../changes/archive/2026-11-10-rule-names/design.md#4-test-finds-the-style-folder-as-vale-does
[rule-setting-7]: ../../changes/archive/2026-11-11-rule-setting/design.md#7-what-stays-different
[settings-2]: ../../changes/archive/2026-11-11-package-settings/design.md#2-the-files-name-and-place-in-the-package
[settings-3]: ../../changes/archive/2026-11-11-package-settings/design.md#3-which-keys-a-package-may-set-and-nested-packages
[settings-4]: ../../changes/archive/2026-11-11-package-settings/design.md#4-a-styles-list-under-a-glob-the-package-also-writes
[settings-5]: ../../changes/archive/2026-11-11-package-settings/design.md#5-scalars-the-project-wins
[settings-6]: ../../changes/archive/2026-11-11-package-settings/design.md#6-order-across-packages
[settings-7]: ../../changes/archive/2026-11-11-package-settings/design.md#7-errors-name-the-package-file
[spelling-1]: ../../changes/archive/2026-11-10-spelling-and-affix-checks/design.md#1-spelling-reads-words-as-vales-tokenizer-cuts-them
[spelling-2]: ../../changes/archive/2026-11-10-spelling-and-affix-checks/design.md#2-dictionaries-and-lists-stay-inside-the-package
[spelling-3]: ../../changes/archive/2026-11-10-spelling-and-affix-checks/design.md#3-fields-vale-reads-and-never-uses
[spelling-4]: ../../changes/archive/2026-11-10-spelling-and-affix-checks/design.md#4-affix-expands-before-compiling
[spelling-goals]: ../../changes/archive/2026-11-10-spelling-and-affix-checks/design.md#goals--non-goals
[splitter-5]: ../../changes/archive/2026-11-10-sentence-splitter/design.md#5-evaluation-against-gold-boundaries-instead-of-vale
[styles-1]: ../../changes/archive/2026-11-11-vale-styles/design.md#1-the-loader-resolves-inheritance
[styles-4]: ../../changes/archive/2026-11-11-vale-styles/design.md#4-a-rule-hakkiri-cannot-run-is-reported-apart
[test-2]: ../../changes/archive/2026-11-10-test-command/design.md#2-a-case-is-a-small-record
[test-5]: ../../changes/archive/2026-11-10-test-command/design.md#5-finding-cases
[test-7]: ../../changes/archive/2026-11-10-test-command/design.md#7-the-report-and-the-exit-status
[tree-1]: ../../changes/archive/2026-11-10-document-tree/design.md#1-markdown-rs-parses-markdown-and-mdx
[vocabularies-3]: ../../changes/archive/2026-11-11-vocabularies/design.md#3-a-vocabulary-is-a-folder-in-a-loaded-package
[vocabularies-4]: ../../changes/archive/2026-11-11-vocabularies/design.md#4-a-line-of-a-list-is-a-pattern
[vocabularies-8]: ../../changes/archive/2026-11-11-vocabularies/design.md#8-what-stays-different
[vocabulary-rules-1]: ../../changes/archive/2026-11-11-vocabulary-rules/design.md#1-terms-and-avoid-are-rules-of-a-style-named-vale
[vocabulary-rules-4]: ../../changes/archive/2026-11-11-vocabulary-rules/design.md#4-the-style-vale-needs-a-vocabulary-term

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

### Requirement: A sentence rule in a comment is read otherwise than Vale reads it

hakkiri SHALL list here the place where it reads a file otherwise than Vale at 66bbb85 that the probe of the ignores and the comment controls left.

#### Scenario: A sentence rule in a comment

- **WHEN** a doc comment of a source file holds one sentence and a rule has `scope: sentence`
- **THEN** hakkiri runs the rule on the sentence, and Vale runs it on none, since it reads the comment as plain lines ([block-token-ignores 5][block-token-ignores-5])

[block-token-ignores-2]: ../../changes/archive/2026-11-12-block-token-ignores/design.md#2-what-a-match-turns-into-is-what-vale-gives
[block-token-ignores-3]: ../../changes/archive/2026-11-12-block-token-ignores/design.md#3-a-source-files-comments-take-the-patterns
[block-token-ignores-4]: ../../changes/archive/2026-11-12-block-token-ignores/design.md#4-patterns-are-checked-when-the-config-loads
[block-token-ignores-not-ported]: ../../changes/archive/2026-11-12-block-token-ignores/design.md#not-ported
[block-token-ignores-5]: ../../changes/archive/2026-11-12-block-token-ignores/design.md#5-a-read-that-differs-from-vales

### Requirement: A message verb with no value differs from Vale

hakkiri SHALL read a `%s` or `%[n]s` of a message that has no value to take as nothing, where Vale 3.26.0 prints Go's error text for it.

#### Scenario: A verb past the values

- **WHEN** a rule has `message: "[%[3]s][%[4]s][%[0]s][%s]"` and two values
- **THEN** hakkiri prints `[][][][]`, and Vale prints `[][%!s(BADINDEX)][%!s(BADINDEX)][%!s(MISSING)]`, since its message gets one empty value more than the check gives ([indexed-verbs 2][indexed-verbs-2])

[indexed-verbs-2]: ../../changes/archive/2026-11-12-indexed-verbs/design.md#2-a-verb-with-no-value-is-nothing

### Requirement: The end of a match that wraps a line differs from Vale

hakkiri SHALL name the line and column of a match's last character, where Vale 3.26.0 prints the first column plus the number of characters of the match, line breaks included.

#### Scenario: A match across a line break

- **WHEN** a rule matches `mea Culpa` where `mea` ends line 1 and `Culpa` starts line 2, with `mea` at column 5
- **THEN** hakkiri ends the alert on line 2, column 5, and Vale ends it at column 13 of line 1, which is past the end of the line ([match-end 1][match-end-1])

[match-end-1]: ../../changes/archive/2026-11-12-match-end/design.md#1-the-end-names-the-last-character

### Requirement: Known gaps are listed apart from what differs by choice

This spec SHALL list each thing Vale 3.26.0 does that hakkiri does not yet do, apart from the intended differences, with its queue row.  
Each gap SHALL be a requirement of its own, named "<gap> is a known gap", so a pull request that closes one removes only that requirement.  
A pull request that closes a gap SHALL record any difference the closing leaves as an intended difference.

#### Scenario: A pull request closes a gap

- **WHEN** a pull request closes the gap that a requirement below names
- **THEN** the same pull request removes that requirement, and adds a scenario to the intended differences when hakkiri still differs

### Requirement: The built-in spelling rule is a known gap

hakkiri SHALL differ from Vale 3.26.0 as the scenario below says, until the queue row it names closes.

#### Scenario: The built-in spelling rule

- **WHEN** a project enables Vale's `Vale.Spelling`, which ships with a default ignore list
- **THEN** hakkiri has no such rule, and a style names its own dictionary and lists (row [HKR-026][ledger], [spelling goals][spelling-goals])

### Requirement: A YAML vocabulary is a known gap

hakkiri SHALL differ from Vale 3.26.0 as the scenario below says, until the queue row it names closes.

#### Scenario: A YAML vocabulary

- **WHEN** a vocabulary folder holds `vocab.yml` with `accept` and `reject` entries, and options such as `regex`, `ignorecase` and `flags`
- **THEN** hakkiri reads `accept.txt` and `reject.txt` only, and Vale reads the YAML file too (row [HKR-041][ledger], [vocabularies 8][vocabularies-8])

### Requirement: The Markdown parser is a known gap

hakkiri SHALL differ from Vale 3.26.0 as the scenario below says, until the queue row it names closes.

#### Scenario: The Markdown parser

- **WHEN** a Markdown file holds syntax that hakkiri's parser and Vale's converter read differently
- **THEN** no such difference is measured yet, so the cases are unknown (row [HKR-027][ledger], [tree 1][tree-1])

### Requirement: A script rule is a known gap

hakkiri SHALL differ from Vale 3.26.0 as the scenario below says, until the queue row it names closes.

#### Scenario: A script rule

- **WHEN** a rule has `extends: script`
- **THEN** hakkiri runs the rest of its style and fails the run for the rule until the config sets it `off`, and Vale runs its Tengo program (row [HKR-036][ledger], [styles 4][styles-4])

### Requirement: A formula outside the listed syntax is a known gap

hakkiri SHALL differ from Vale 3.26.0 as the scenario below says, until the queue row it names closes.

#### Scenario: A formula outside the listed syntax

- **WHEN** a `metric` rule has the formula `words > 1 ? 7.0 : 0.0`
- **THEN** hakkiri refuses it, since it reads numbers, variables, operators, `math` functions and `round`, and Vale evaluates one Tengo expression and reports 7.00 (row [HKR-037][ledger], [engine 14][engine-14])

### Requirement: The built-in repetition rule is a known gap

hakkiri SHALL differ from Vale 3.26.0 as the scenario below says, until the queue row it names closes.

#### Scenario: The built-in repetition rule

- **WHEN** a glob lists the style `Vale`, which holds `Vale.Repetition`
- **THEN** hakkiri has no such style and refuses the name, and Vale reports a repeated word (row [HKR-038][ledger])

### Requirement: A doc selector or a View is a known gap

hakkiri SHALL differ from Vale 3.26.0 as the scenario below says, until the queue row it names closes.

#### Scenario: A doc selector or a View

- **WHEN** a rule has `scope: doc(h1)`, a section names a View, or a case sets `view`
- **THEN** hakkiri runs the rest of the style and fails the run for the rule until the config sets it `off`.
  It refuses the case field, where Vale selects by CSS over the rendered HTML (row [HKR-039][ledger], [test 2][test-2])

### Requirement: A Vale configuration file is a known gap

hakkiri SHALL differ from Vale 3.26.0 as the scenario below says, until the queue row it names closes.

#### Scenario: A Vale configuration file

- **WHEN** a project holds a `.vale.ini` and runs `hakkiri import vale`
- **THEN** hakkiri has no such command yet, and the config is written by hand (row [HKR-035][ledger])

### Requirement: An action is a known gap

hakkiri SHALL differ from Vale 3.26.0 as the scenario below says, until the queue row it names closes.

#### Scenario: An action

- **WHEN** a rule holds an `action`
- **THEN** hakkiri loads it and applies nothing, and Vale offers the fix (row [HKR-011][ledger])

### Requirement: Comment controls differ from Vale's in these ways

hakkiri SHALL read the comments that turn rules off and on as Vale does in a Markdown file, apart from the scenarios below, and SHALL read the forms of its own as well.

#### Scenario: Forms hakkiri adds

- **WHEN** a Markdown file holds `<!-- hakkiri Team.Words: warning -->` or `<!-- hakkiri Team.Words: off -->`
- **THEN** hakkiri sets the level of the rule from there on, or turns it off, and Vale reads neither form

#### Scenario: A source file

- **WHEN** a doc comment of a Rust, TypeScript or JavaScript file holds an HTML comment that turns rules off
- **THEN** hakkiri turns nothing off, and Vale reads a comment as plain lines, so it turns nothing off either ([comment-controls 4][comment-controls-4])

#### Scenario: A directive quoted across a line break

- **WHEN** a code span reads `<!-- vale off` on one line and `-->` on the next, and a real `vale off` comment comes after it
- **THEN** hakkiri hides the text after the real comment, and Vale also hides the text after the code span, since it takes the comment's place from the first `vale off` that its search finds ([comment-controls 8][comment-controls-8])

#### Scenario: An alert in an element that holds a comment

- **WHEN** a `strong` element holds a comment, and its text holds a match
- **THEN** hakkiri places the alert on the match, and Vale places it on the comment ([comment-controls 8][comment-controls-8])

#### Scenario: The styles of a file

- **WHEN** a comment limits the styles to `Other`
- **THEN** hakkiri turns off the styles of the rules the config runs on the file, and Vale turns off its `BasedOnStyles` ([comment-controls 3][comment-controls-3])

[comment-controls-8]: ../../changes/archive/2026-11-12-comment-controls/design.md#8-a-comment-that-vale-places-wrongly
[comment-controls-3]: ../../changes/archive/2026-11-12-comment-controls/design.md#3-a-style-comment-reads-the-styles-of-the-rules
[comment-controls-4]: ../../changes/archive/2026-11-12-comment-controls/design.md#4-a-source-file-holds-no-comment-control

### Requirement: A walk and a named file differ from Vale's

hakkiri SHALL skip the files git ignores in a walk and SHALL warn about a named file that no glob selects, where Vale 3.26.0 does neither.

#### Scenario: An ignored build folder

- **WHEN** `.gitignore` holds `build/` and the folder holds a file a glob selects
- **THEN** hakkiri skips it, and Vale lints it, because Vale skips only its default ignored folders and reads no git file ([gitignore 6][gitignore-6])

#### Scenario: A named file with no glob

- **WHEN** `hakkiri check README.md` runs and no glob matches it
- **THEN** hakkiri warns that no glob selects it, and Vale lints it with no rule and reports nothing ([gitignore 6][gitignore-6])

[gitignore-6]: ../../changes/archive/2026-11-12-gitignore-walk/design.md#6-what-stays-different-from-vale

### Requirement: Readers differ from Vale in what a file with no reader gets

hakkiri SHALL read a selected file that no reader reads as plain text, skip a binary file, and map a format as each scenario below says, where Vale 3.26.0 does otherwise.

#### Scenario: A manuscript

- **WHEN** a `.txt` file holds chapter lines
- **THEN** hakkiri reads paragraphs, and Vale reads a document whose chapter lines are headings ([plain-text 1][plain-text-1])

#### Scenario: A binary file

- **WHEN** a glob selects a file that holds a NUL byte
- **THEN** hakkiri skips it, and Vale has no such test and reads it as lines ([plain-text 5][plain-text-5])

#### Scenario: The extension of a mapped file

- **WHEN** `formats` maps `txt: md` and a rule has `scope: text.md`
- **THEN** hakkiri selects the text of `a.txt`, and Vale compares the scope's extension with the file's own, `.txt`, and selects nothing; both select it for `text.txt` ([plain-text 6][plain-text-6])

#### Scenario: A plain-text file with markup

- **WHEN** `notes.cfg` holds a fenced block, a glob selects it and no `formats` entry maps it
- **THEN** hakkiri reads the fence as text, as Vale reads a file of lines ([plain-text 1][plain-text-1])

#### Scenario: A format in a package and in the project

- **WHEN** a package maps `txt: md` and the project maps `txt: text`
- **THEN** hakkiri reads the project's value, and Vale takes the first value for `[formats]`, so the package wins ([package settings 5][settings-5])

[plain-text-1]: ../../changes/archive/2026-11-12-plain-text-files/design.md#1-plain-text-is-one-raw-root-of-paragraphs
[plain-text-5]: ../../changes/archive/2026-11-12-plain-text-files/design.md#5-a-binary-file-is-skipped
[plain-text-6]: ../../changes/archive/2026-11-12-plain-text-files/design.md#6-a-mapped-file-has-two-extensions

### Requirement: Doc line comments join across blank lines

hakkiri SHALL join Rust doc line comments of the same type across blank lines, where Vale 3.26.0 joins a line comment only when it follows the previous one directly.

#### Scenario: A fence across a blank line

- **WHEN** a Rust file holds `/// ```sh`, an empty line, `/// curl it` and `/// ```` `
- **THEN** hakkiri reads `curl it` as code, and Vale reads two comments, so a `text` rule sees `curl it` as prose ([doc-fence 1][doc-fence-1])

[doc-fence-1]: ../../changes/archive/2026-11-12-doc-fence/design.md#1-doc-line-comments-join-across-blank-lines
