## REMOVED Requirements

### Requirement: The built-in spelling rule is a known gap

**Reason**: The style `Vale` now holds `Vale.Spelling`, so the gap closes.

**Migration**: Read the requirement "The built-in spelling rule differs from Vale's in these ways" under the ADDED section for what still differs.

### Requirement: Ported readers differ from Vale in detail

**Reason**: One of its scenarios, an email address that hakkiri reads no word in, no longer holds, since hakkiri keeps a bare email address in the text as Vale does.

**Migration**: Read the requirement "Ported readers differ from Vale in these ways" under the ADDED section, which holds the other scenarios unchanged.

## ADDED Requirements

### Requirement: The built-in spelling rule differs from Vale's in these ways

hakkiri SHALL run `Vale.Spelling` as Vale does, apart from the scenarios below and the differences of a `spelling` rule that this spec already lists.

#### Scenario: Several packages

- **WHEN** a project loads two packages that each hold a `config/ignore` folder or a `config/dictionaries` folder
- **THEN** hakkiri reads both, in order, and Vale reads one styles path ([vale-spelling 2][vale-spelling-2])

#### Scenario: A code span in a code comment

- **WHEN** a Rust comment holds `` `blorftCode` `` and `custom` turns the built-in filters off
- **THEN** hakkiri reads no word in the code span, and Vale reports the span with its backticks as one word ([vale-spelling 5][vale-spelling-5])

#### Scenario: The name after a param tag

- **WHEN** a TypeScript comment holds `@param nme the nme of it`
- **THEN** hakkiri reads no word for the name `nme`, and Vale reports it, and reports `@param` when `custom` turns the filters off ([vale-spelling 5][vale-spelling-5])

### Requirement: Ported readers differ from Vale in these ways

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

#### Scenario: Two swap keys expand to the same text

- **WHEN** an `affix` rule's `swap` has two keys whose forms expand to the same text
- **THEN** hakkiri keeps both in the order written, so the first gives the replacement, and Vale keeps the key its map visits last, which can change from run to run ([spelling 4][spelling-4])

#### Scenario: A language other than English

- **WHEN** a project checks text in a language other than English
- **THEN** hakkiri does not support it, which the spelling change states as a non-goal, and Vale reads a dictionary for the language ([spelling goals][spelling-goals])

[vale-spelling-2]: ../../design.md#2-the-built-in-rule-reads-the-settings-of-every-package
[vale-spelling-5]: ../../design.md#5-what-the-probe-shows-and-hakkiri-differs-on
[case-3]: ../../../2026-11-10-case-and-sequence-checks/design.md#3-sequence-walks-tokens-around-an-anchor
[case-5]: ../../../2026-11-10-case-and-sequence-checks/design.md#5-tags-come-from-a-port-of-proses-tagger-and-model
[case-6]: ../../../2026-11-10-case-and-sequence-checks/design.md#6-spelling-ports-vales-hunspell-reader
[spelling-1]: ../../../2026-11-10-spelling-and-affix-checks/design.md#1-spelling-reads-words-as-vales-tokenizer-cuts-them
[spelling-4]: ../../../2026-11-10-spelling-and-affix-checks/design.md#4-affix-expands-before-compiling
[spelling-goals]: ../../../2026-11-10-spelling-and-affix-checks/design.md#goals--non-goals
