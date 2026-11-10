## ADDED Requirements

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

hakkiri SHALL finish a run on each input below in bounded time and memory, where Vale 3.26.0 panics, runs out of memory, or takes cubic or exponential time.

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

### Requirement: Known gaps are listed apart

This spec SHALL list each thing Vale 3.26.0 does that hakkiri does not yet do, apart from the intended differences, with its queue row.  
A pull request that closes a gap SHALL remove its scenario, and record any difference the closing leaves as an intended difference.

#### Scenario: Text inside an HTML block

- **WHEN** a Markdown file holds prose inside an HTML block
- **THEN** hakkiri reads none of it, and Vale lints it with the scope of its block
- **AND** Vale also lints the text after a closed raw-text tag or a `--!>`
- **AND** Vale's entity unescaping, masking of `code`, `tt`, `kbd` and `samp`, and handling of `pre` are not yet probed (row [HKR-021][ledger], [case 9][case-9])

#### Scenario: The built-in spelling rule

- **WHEN** a project enables Vale's `Vale.Spelling`, which ships with a default ignore list
- **THEN** hakkiri has no such rule, and a style names its own dictionary and lists (row [HKR-026][ledger], [spelling goals][spelling-goals])

#### Scenario: Vocabularies

- **WHEN** a rule sets `vocab`, or a project lists accepted and rejected terms
- **THEN** hakkiri loads the field and reads no vocabulary, and Vale adds the accepted terms to the exceptions and generates the `Vale.Terms` and `Vale.Avoid` rules (row [HKR-019][ledger], [case 1][case-1])

#### Scenario: The Markdown parser

- **WHEN** a Markdown file holds syntax that hakkiri's parser and Vale's converter read differently
- **THEN** no such difference is measured yet, so the cases are unknown (row [HKR-027][ledger], [tree 1][tree-1])

[bootstrap-8]: ../../../archive/2026-11-09-bootstrap-hakkiri/design.md#8-two-regex-engines-with-a-limit
[case-3]: ../../../archive/2026-11-10-case-and-sequence-checks/design.md#3-sequence-walks-tokens-around-an-anchor
[case-5]: ../../../archive/2026-11-10-case-and-sequence-checks/design.md#5-tags-come-from-a-port-of-proses-tagger-and-model
[case-6]: ../../../archive/2026-11-10-case-and-sequence-checks/design.md#6-spelling-ports-vales-hunspell-reader
[engine-8]: ../../../archive/2026-11-10-rule-engine/design.md#8-two-regex-engines-and-a-backtrack-limit
[spelling-1]: ../../../archive/2026-11-10-spelling-and-affix-checks/design.md#1-spelling-reads-words-as-vales-tokenizer-cuts-them
[spelling-4]: ../../../archive/2026-11-10-spelling-and-affix-checks/design.md#4-affix-expands-before-compiling
[case-1]: ../../../archive/2026-11-10-case-and-sequence-checks/design.md#1-case-conversion-ports-proses-converters
[case-9]: ../../../archive/2026-11-10-case-and-sequence-checks/design.md#9-a-block-of-text-has-the-scope-of-one-element
[spelling-goals]: ../../../archive/2026-11-10-spelling-and-affix-checks/design.md#goals--non-goals
[tree-1]: ../../../archive/2026-11-10-document-tree/design.md#1-markdown-rs-parses-markdown-and-mdx
[ledger]: ../../../../../tasks.yaml
