# Spec Delta

## Purpose

A `spelling` rule checks each word of the selected text against a dictionary and reports the words it does not hold.  
An `affix` field lets an `existence` token or a `substitution` key name a word with its affix flags.  
Both keep the meaning Vale 3.26.0 gives them.

## ADDED Requirements

### Requirement: spelling reports a word the dictionary lacks

A `spelling` rule SHALL split the selected text into words.  
It SHALL report each word that the rule's dictionaries do not hold, as one alert that spans the word.  
A `%s` in its message SHALL read as the word.  
When the rule loads no other dictionary, the dictionary SHALL be Vale's built-in `en_US-web`.

#### Scenario: A misspelt word

- **WHEN** a rule has `extends: spelling` and `message: "Did you really mean '%s'?"`, and the input is `We blorft mail.`
- **THEN** hakkiri reports one alert, with the matched text `blorft` and the message `Did you really mean 'blorft'?`

#### Scenario: A word in a code span

- **WHEN** the same rule reads ``Run `blorft` now.``
- **THEN** hakkiri reports no alert

### Requirement: Built-in filters skip names and codes

Unless the rule sets `custom: true`, three kinds of word SHALL be skipped.  
These are a word that starts with a capital and has another capital after a lower-case letter, and a word that ends in capitals.  
The third is a word that holds a character other than an ASCII letter, `_` or `'`.

A word that matches one of the rule's `filters` SHALL also be skipped.

#### Scenario: A name, an acronym and a version

- **WHEN** the rule above reads `Ask FooBar and ACME about v2.`
- **THEN** hakkiri reports no alert

#### Scenario: Built-in filters off

- **WHEN** the rule has `custom: true` and reads `Ask FooBar now.`
- **THEN** hakkiri reports one alert, with the matched text `FooBar`

#### Scenario: A filter of the rule

- **WHEN** the rule has `filters: ['^hak']` and reads `Run hakkiri now.`
- **THEN** hakkiri reports no alert

### Requirement: ignore adds word lists

Each entry of `ignore` SHALL name a file of accepted words, one per line.  
The file SHALL be found at that path in the package that holds the rule, or under the package's `config/ignore` folder.

#### Scenario: A project term

- **WHEN** the rule has `ignore: [terms.txt]`, its package holds `config/ignore/terms.txt` with the line `hakkiri`, and the input is `Run hakkiri now.`
- **THEN** hakkiri reports no alert

### Requirement: split checks the parts of an identifier

With `split: true`, a word made of parts, such as `blorftMessage` or `blorft_message`, SHALL be checked part by part.  
A misspelt part of three letters or more that is not in capitals SHALL be reported at its own position.

#### Scenario: An identifier without split

- **WHEN** the rule above reads `Call blorftMessage now.`
- **THEN** hakkiri reports one alert, with the matched text `blorftMessage`

#### Scenario: An identifier with split

- **WHEN** the rule has `split: true` and reads `Call blorftMessage now.`
- **THEN** hakkiri reports one alert at line 1, column 6, with the matched text `blorft`

### Requirement: Vale's casing and code messages

When the dictionary holds the word only with more capitals inside it, the message SHALL name the dictionary form.  
It SHALL read `Use '<dictionary form>' instead of '<word>'.`  
Otherwise, without `split`, a word shaped like an identifier in a Markdown or MDX file SHALL get the message `'<word>' looks like code; format it as code.`

#### Scenario: A casing mistake

- **WHEN** the rule above reads `Push to github.`
- **THEN** hakkiri reports one alert, with the message `Use 'GitHub' instead of 'github'.`

#### Scenario: An identifier in Markdown

- **WHEN** a Markdown file reads `Set apiVersion first.`
- **THEN** hakkiri reports one alert, with the message `'apiVersion' looks like code; format it as code.`

### Requirement: dictionaries come from the package

`dictionaries` SHALL name dictionaries read as `<name>.aff` and `<name>.dic` from the `config/dictionaries` folder of the rule's package.  
When `dicpath` is set, they SHALL be read from that folder inside the package instead.  
`append: true` SHALL add the built-in dictionary to the ones the rule loads.

`aff` and `dic` SHALL name one dictionary's two files inside the package.  
hakkiri SHALL refuse a rule whose dictionary is missing or whose path leaves its package.

#### Scenario: A team dictionary

- **WHEN** the rule has `dictionaries: [team]` and `append: true`, its package holds `config/dictionaries/team.aff` and a `team.dic` with the word `hakkiri`, and the input is `Run hakkiri on blorft.`
- **THEN** hakkiri reports one alert, with the matched text `blorft`

#### Scenario: A missing dictionary

- **WHEN** rule `Team.Spelling` has `dictionaries: [absent]` and its package holds no `absent.dic`
- **THEN** hakkiri reports an error naming `Team.Spelling` and `absent`, and exits with a failure status

### Requirement: affix expands a word with flags

When an `existence` or `substitution` rule sets `affix`, each token or key part written `word/FLAGS` SHALL match the word and each form the flags name in that affix file.  
`affix: en_US-web` SHALL name the built-in affix file, and any other name SHALL be read from the package's `config/dictionaries`.

#### Scenario: Every form of a verb

- **WHEN** a rule has `extends: substitution`, `message: "Use '%s' instead of '%s'."`, `affix: en_US-web` and `swap: {utilize/DSG: use}`, and the input is `We utilized it.`
- **THEN** hakkiri reports one alert, with the message `Use 'use' instead of 'utilized'.`

#### Scenario: An affix file that is missing

- **WHEN** rule `Team.Forms` has `extends: existence`, `affix: absent` and `tokens: [run/G]`
- **THEN** hakkiri reports an error naming `Team.Forms` and `absent.aff`, and exits with a failure status
