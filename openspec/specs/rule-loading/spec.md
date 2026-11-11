# rule-loading Specification

## Purpose

Loading turns the rule files of a style folder into rules the engine can run.  
It refuses a rule it cannot run, with an error that names the file, the field and the line.

## Requirements

### Requirement: A style folder gives one rule per file

hakkiri SHALL load every `.yml` file under a style folder as one rule, in a folder that does not start with `.` or `_`, and SHALL ignore every other file there.  
A file named `*.test.yml` holds cases and is no rule.

#### Scenario: A folder with a rule and a README

- **WHEN** a style folder `Team` holds `Filler.yml` and `README.md`
- **THEN** hakkiri loads one rule, `Team.Filler`

#### Scenario: Files that are no rules

- **WHEN** the style folder `Team` holds `Filler.test.yml`, `_draft/Old.yml` and `.hidden/Old.yml`, none with an `extends`
- **THEN** hakkiri loads no rule from them and reports no error

### Requirement: A rule needs extends and message

hakkiri SHALL refuse a rule file that has no `extends` or no `message`, and report an error that names the file and the missing field.

#### Scenario: A rule with no message

- **WHEN** `Team/Short.yml` holds `extends: existence` and `tokens: [very]` only
- **THEN** hakkiri reports an error naming `Team/Short.yml` and `message`, and exits with a failure status

### Requirement: An unknown extends is refused

hakkiri SHALL refuse a rule whose `extends` names neither a check type nor a rule, and report an error that names the file and the value.

#### Scenario: A check type Vale does not define

- **WHEN** `Team/Grammar.yml` holds `extends: grammar`
- **THEN** hakkiri reports an error naming `Team/Grammar.yml` and `grammar`, and exits with a failure status

### Requirement: An unknown field is refused

hakkiri SHALL refuse a rule that holds a field its check type does not define.  
The error SHALL name the field and its line.

#### Scenario: token in an existence rule

- **WHEN** an `existence` rule holds `token: very` on line 3, where the type defines `tokens`
- **THEN** hakkiri reports an error naming `token` and line 3, and exits with a failure status

### Requirement: A field of the wrong type is refused

hakkiri SHALL refuse a rule whose field holds a value of the wrong type, and report an error that names the field and its line.  
A number written as a string, such as `max: '3'`, SHALL read as the number.

#### Scenario: A count that is not a number

- **WHEN** an `occurrence` rule holds `max: many` on line 4
- **THEN** hakkiri reports an error naming `max` and line 4

#### Scenario: A number in quotes

- **WHEN** an `occurrence` rule holds `max: '3'`
- **THEN** hakkiri loads the rule with a `max` of 3

### Requirement: level takes three values

A rule's `level` SHALL be `suggestion`, `warning` or `error`, and SHALL be `warning` when the rule sets none.  
hakkiri SHALL refuse any other value and name the file and the field.

#### Scenario: A rule with no level

- **WHEN** an `existence` rule with `tokens: [very]` and no `level` reads `It is very good.`
- **THEN** hakkiri reports one warning

#### Scenario: A level Vale does not define

- **WHEN** a rule holds `level: info`
- **THEN** hakkiri reports an error naming the file and `level`, and exits with a failure status

### Requirement: A pattern that does not compile is refused

hakkiri SHALL refuse a rule with a pattern that neither regex engine compiles, and report an error that names the rule and the pattern.

#### Scenario: An unclosed group

- **WHEN** `Team.Broken` has `extends: existence` and `tokens: ['(unclosed']`
- **THEN** hakkiri reports an error naming `Team.Broken` and `(unclosed`, and exits with a failure status

### Requirement: A pattern with nothing to match is refused

hakkiri SHALL refuse an `existence` rule whose `tokens` and `raw` hold nothing to match, and a `metric` rule that lists `tokens` or `raw` holding nothing, because such a pattern matches at every position.  
The error SHALL name the file, the line, `tokens` and `raw`.

#### Scenario: An existence rule with no tokens

- **WHEN** `Team/Empty.yml` holds `extends: existence` and `message: m` only
- **THEN** hakkiri reports an error naming `Team/Empty.yml`, line 1, `tokens` and `raw`, and exits with a failure status

#### Scenario: Tokens of blanks only

- **WHEN** an `existence` rule holds `tokens: ['', '  ']` and no `raw`
- **THEN** hakkiri reports the same error

#### Scenario: A metric with an empty raw

- **WHEN** a `metric` rule holds `raw: ['']` and no `tokens`
- **THEN** hakkiri reports the same error

### Requirement: A YAML error names the line

hakkiri SHALL refuse a rule file that is not valid YAML, and report an error that names the file and the line of the fault.

#### Scenario: A tab in the indentation

- **WHEN** `Team/Tabs.yml` holds a tab character in the indentation of line 5
- **THEN** hakkiri reports an error naming `Team/Tabs.yml` and line 5

### Requirement: An action loads and is not applied

hakkiri SHALL load a rule's `action` field, carry it on the rule's alerts, and change no text with it.

#### Scenario: An existence rule with an action

- **WHEN** a rule has `extends: existence`, `message: "Remove '%s'."`, `tokens: [very]` and `action: {name: remove}`, and the input is `It is very good.`
- **THEN** hakkiri reports one alert with the message `Remove 'very'.`, and the file is unchanged

### Requirement: A rule is named by its path

hakkiri SHALL name a rule by its style folder, each folder between it and the file, and the file name up to its first dot, joined by dots, as Vale does.  
A name with `[` or `]` SHALL be refused when a glob lists the style.  
`check`, `test` and `test --coverage` SHALL give a file one name.

#### Scenario: A dotted file name

- **WHEN** the style folder `Team` holds `Terms.custom.yml` and `Foo.v2.yml`
- **THEN** the rules are `Team.Terms` and `Team.Foo`, and a config key `Team.Terms: off` turns the first one off

#### Scenario: A nested rule

- **WHEN** the style folder `Team` holds `Sub/Bar.yml` and `Sub/Deep/Baz.yml`
- **THEN** the rules are `Team.Sub.Bar` and `Team.Sub.Deep.Baz`, in `check` and in the `test` cases that name them

#### Scenario: A nested rule under coverage

- **WHEN** the config has the package `styles`, `styles/Team/Sub/Deep/Bar.yml` flags `perhaps`, and a case through the config expects `Team.Sub.Deep.Bar`
- **THEN** `test --coverage` passes the case and lists no uncovered rule

#### Scenario: A bracket in a name

- **WHEN** the style folder `Team` holds `Weird[max].yml`, and a glob lists `Team`
- **THEN** hakkiri refuses it with an error that names the file and the rule name `Team.Weird[max]`

### Requirement: Two files with one name are refused

hakkiri SHALL refuse a style folder in which two rule files give one rule name, when a glob lists the style, with an error that names the rule and both files.  
A style no glob lists SHALL load with no error, as in Vale.

#### Scenario: Terms.yml and Terms.custom.yml

- **WHEN** the style folder `Team` holds `Terms.yml` and `Terms.custom.yml`, and a glob lists `Team`
- **THEN** hakkiri exits with a failure status, and the error names `Team.Terms`, `Terms.yml` and `Terms.custom.yml`

#### Scenario: A clash or a bracket in a style no glob lists

- **WHEN** the style folder `Team` holds `Terms.yml` and `Terms.custom.yml`, or `Weird[max].yml`, and no glob lists `Team`
- **THEN** hakkiri lints with the other styles and exits as it would without `Team`

### Requirement: A rule may extend another rule

A rule whose `extends` holds a dot SHALL start from the rule that name gives and lay its own fields on top, as Vale 3.26.0 does.  
The first segment of the name SHALL be a style folder of any loaded package, and the rest SHALL be folders and a file name, in a nested folder or a `_` folder too.  
The parent's `tests` SHALL NOT carry over.

#### Scenario: A child takes the fields of its parent

- **WHEN** `Kid/Child.yml` holds `extends: Base.Parent`, `message: "Child: found %s"` and `level: error`, `Base/Parent.yml` flags `alpha`, `beta` and `gamma`, and the input is `alpha beta gamma delta swapme`
- **THEN** hakkiri reports three errors from `Kid.Child`, with the message `Child: found alpha` and the two after it

#### Scenario: A parent in a nested folder or a `_` folder

- **WHEN** a rule holds `extends: Base.Deep.Parent`, or `extends: Base._shared.Parent`
- **THEN** hakkiri reads the rule from `Base/Deep/Parent.yml`, or from `Base/_shared/Parent.yml`, which no style loads as a rule

#### Scenario: A parent in another package

- **WHEN** the config lists the packages `one` and `two`, `one/Kid/Child.yml` holds `extends: Base.Parent`, and `two/Base/Parent.yml` exists
- **THEN** hakkiri finds the parent in `two`

#### Scenario: A fetched package

- **WHEN** the config pins a Vale package by `url` and `sha256`, and its rule `Direct.Length` extends `Std.Readability.SentenceLength`, which the same zip holds
- **THEN** hakkiri reads the package from the cache and loads the rule

#### Scenario: A chain of three

- **WHEN** `Kid/Grand.yml` extends `Kid.Child`, which extends `Base.Parent`
- **THEN** the rule starts from `Base.Parent`, and `Kid.Child` and `Kid.Grand` lay their fields on top in turn

#### Scenario: Cases of the child only

- **WHEN** a parent and its child each hold one case
- **THEN** `hakkiri test` runs one case for each rule, and the child does not run the case of its parent

### Requirement: A key with a plus or a minus edits a field of the parent

`key+` SHALL add its entries to a list of the parent, or merge them into a mapping of the parent with the child's entries winning.  
`key-` SHALL remove entries from a list by their text, or keys from a mapping by name.  
A bare key SHALL replace the parent's value.

#### Scenario: An entry added to a list

- **WHEN** a child of a rule that flags `alpha`, `beta` and `gamma` holds `tokens+: [delta]`
- **THEN** the child flags all four words, with the message of the parent

#### Scenario: An entry removed from a list

- **WHEN** the child holds `tokens-: [beta]`
- **THEN** the child flags `alpha` and `gamma`

#### Scenario: A mapping edited

- **WHEN** a child of a rule with `swap: {swapme: swapped, delta: delta2}` holds `swap+: {alpha: omega}` and `swap-: [delta]`
- **THEN** the child swaps `alpha` for `omega` and `swapme` for `swapped`, and not `delta`

### Requirement: A chain Vale refuses is refused

hakkiri SHALL refuse a chain of rules that Vale 3.26.0 refuses, with an error that names the rule file and the field.  
A rule read through a parent SHALL name no line.

#### Scenario: A key written two ways

- **WHEN** a child holds `tokens` and `tokens+`
- **THEN** hakkiri reports an error naming the file and `tokens+`: it both replaces `tokens` and edits it, so pick one, and exits 2

#### Scenario: An entry the parent lacks

- **WHEN** a child holds `tokens-: [missing]`, and the parent's `tokens` has no `missing`
- **THEN** hakkiri reports that the parent has no entry `missing` to remove, and exits 2

#### Scenario: A cycle

- **WHEN** `Kid/Child.yml` holds `extends: Kid.Child`
- **THEN** hakkiri reports an inheritance cycle, and exits 2

#### Scenario: A chain over ten rules

- **WHEN** a rule extends a chain of twelve rules
- **THEN** hakkiri reports that the chain is over 10 rules deep, and exits 2

#### Scenario: A parent that does not exist

- **WHEN** a rule holds `extends: Base.Absent` and no package holds `Base/Absent.yml`
- **THEN** hakkiri reports that `Base.Absent` is not on the search path, and exits 2

#### Scenario: A built-in parent

- **WHEN** a rule holds `extends: Vale.Terms`
- **THEN** hakkiri reports that `Vale.Terms` is a built-in rule with no file to extend, and exits 2

#### Scenario: A parent name that is a path

- **WHEN** a rule holds `extends: /etc/x.y` or `extends: Std..Rule`
- **THEN** hakkiri reports that it is not a rule name, says a rule name is a style, a dot and a rule such as `Std.Rule`, reads no file for it, and exits 2

### Requirement: A rule hakkiri cannot run loads no rule

A rule that is valid Vale and that hakkiri cannot run, a rule that extends `script` or whose `scope` is a `doc()` selector, SHALL give no rule.  
It SHALL NOT stop the other rules of its style from loading.  
The loader SHALL keep its name and its file, so a config can turn it `off`.

#### Scenario: A script rule beside a rule that runs

- **WHEN** `Kid/Cannot.yml` holds `extends: script` and `Kid/Parent.yml` flags `alpha`
- **THEN** the style `Kid` loads the rule `Kid.Parent` and holds `Kid.Cannot` as a rule it cannot run

#### Scenario: A doc selector

- **WHEN** a rule holds `scope: doc(h1)`
- **THEN** the style holds it as a rule it cannot run

#### Scenario: A child of a script rule

- **WHEN** a rule extends a rule that extends `script`
- **THEN** it is a rule hakkiri cannot run too

#### Scenario: Two files for one rule

- **WHEN** two files of a style give one rule name, and the later file is a rule hakkiri cannot run or the earlier one is
- **THEN** the later file replaces the earlier one: the rule runs only if the later file is runnable, and fails the run only if it is not
