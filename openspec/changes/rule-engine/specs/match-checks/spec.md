# Spec Delta

## Purpose

The match checks find patterns in the text a scope selects.  
`existence` reports each match, `substitution` names the word to use instead, and `occurrence` counts matches against a limit.  
Each keeps the meaning Vale 3.26.0 gives it.

## ADDED Requirements

### Requirement: existence tokens match whole words

An `existence` rule SHALL report each match of any of its `tokens`, each bounded by word boundaries.  
With `nonword: true` it SHALL add no word boundaries.

#### Scenario: A word and a longer word

- **WHEN** a rule has `extends: existence`, `message: "'%s'"` and `tokens: [cat]`, and the input is `A cat and a category.`
- **THEN** hakkiri reports one alert, with the matched text `cat`

#### Scenario: Punctuation with nonword

- **WHEN** a rule has `nonword: true` and `tokens: ['!']`, and the input is `Wow! Great!`
- **THEN** hakkiri reports two alerts

### Requirement: raw comes before the tokens

An `existence` rule SHALL join its `raw` patterns and match them before its tokens, or after them when `append` is `true`.

#### Scenario: A word before a token

- **WHEN** a rule has `raw: ['\bvery\s+']` and `tokens: [good]`, and the input is `It is very good.`
- **THEN** hakkiri reports one alert, with the matched text `very good`

### Requirement: An exception holds back a match

A match that holds an entry of `exceptions` as a whole word SHALL give no alert.  
A match inside a multi-word exception SHALL give no alert.

#### Scenario: An adverb list with an exception

- **WHEN** a rule has `tokens: ['\w+ly']` and `exceptions: [only]`, and the input is `It only barely works.`
- **THEN** hakkiri reports one alert, with the matched text `barely`

#### Scenario: A word inside an accepted phrase

- **WHEN** a rule has `tokens: [culpa]` and `exceptions: ['mea culpa']`, and the input is `A mea culpa and a culpa.`
- **THEN** hakkiri reports one alert, at the second `culpa`

#### Scenario: Two phrases that overlap

- **WHEN** a rule has `tokens: [c]` and `exceptions: ['a b', 'b c']`, and the input is `a b c`
- **THEN** hakkiri reports no alert

### Requirement: substitution tries longer keys first

A `substitution` rule SHALL match its `swap` keys as `existence` matches tokens, trying longer keys first.  
Its message SHALL take the swap value, then the matched text.

#### Scenario: A phrase that holds a shorter key

- **WHEN** a rule has `message: "Use '%s' instead of '%s'."` and `swap: {in order to: to, order: sequence}`, and the input is `In order to win.` with `ignorecase: true`
- **THEN** hakkiri reports one alert, with the message `Use 'to' instead of 'In order to'.`

### Requirement: A match already in the swap's form passes

A `substitution` match that equals its swap value, or matches it as a pattern, SHALL give no alert.

#### Scenario: A name in two spellings

- **WHEN** a rule has `ignorecase: true` and `swap: {github: GitHub}`, and the input is `Use GitHub and github.`
- **THEN** hakkiri reports one alert, with the matched text `github`

### Requirement: capitalize follows the match

With `capitalize: true`, a `substitution` rule SHALL capitalize the swap value when the matched text starts with a capital letter.

#### Scenario: A sentence that starts with the key

- **WHEN** a rule has `capitalize: true`, `message: "Use '%s' instead of '%s'."` and `swap: {utilize: use}`, and the input is `Utilize it.` with `ignorecase: true`
- **THEN** hakkiri reports one alert, with the message `Use 'Use' instead of 'Utilize'.`

### Requirement: A swap value can use the key's groups

A `$1` in a swap value SHALL read as the text the first group of its key matched, and so on for each group.

#### Scenario: A verb kept from the match

- **WHEN** a rule has `message: "Use '%s' instead of '%s'."` and `swap: {'(\w+) out of': '$1 from'}`, and the input is `Jump out of it.`
- **THEN** hakkiri reports one alert, with the message `Use 'Jump from' instead of 'Jump out of'.`

### Requirement: occurrence counts the token as written

An `occurrence` rule SHALL count the matches of its `token` in each selected node, with no word boundaries added.  
It SHALL report when the count is over `max` or under `min`.  
Its message and description SHALL read `%d` as `%s`, so the count fills either one.

#### Scenario: Matches inside words

- **WHEN** a rule has `extends: occurrence`, `message: "%d endings"`, `max: 1` and `token: 'ing\b'`, and the input is `Going and coming.`
- **THEN** hakkiri reports one alert, with the message `2 endings`

### Requirement: An occurrence alert points at a match

An alert over `max` SHALL point at the first match that is not a code span.  
An alert under `min` with no match SHALL point at the first word of the node.

#### Scenario: Too few matches

- **WHEN** a rule has `extends: occurrence`, `message: "%d uses"`, `scope: paragraph`, `min: 2` and `token: '\bthe\b'`, and the input is `A cat sat.`
- **THEN** hakkiri reports one alert, with the message `0 uses` and the matched text `A`, at column 1
